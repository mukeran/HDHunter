/*
 * Licensed to the Apache Software Foundation (ASF) under one or more
 * contributor license agreements.  See the NOTICE file distributed with
 * this work for additional information regarding copyright ownership.
 * The ASF licenses this file to You under the Apache License, Version 2.0
 * (the "License"); you may not use this file except in compliance with
 * the License.  You may obtain a copy of the License at
 *
 *      http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

package org.apache.catalina.filters;

import java.io.IOException;
import java.time.Instant;
import java.util.Enumeration;
import java.util.Map;

import jakarta.servlet.FilterChain;
import jakarta.servlet.FilterConfig;
import jakarta.servlet.ServletContext;
import jakarta.servlet.ServletException;

import org.junit.Assert;
import org.junit.Test;

import org.apache.catalina.Context;
import org.apache.catalina.LifecycleException;
import org.apache.catalina.filters.TestRemoteIpFilter.MockFilterChain;
import org.apache.catalina.filters.TestRemoteIpFilter.MockHttpServletRequest;
import org.apache.catalina.startup.Tomcat;
import org.apache.catalina.startup.TomcatBaseTest;
import org.apache.tomcat.unittest.TesterResponse;
import org.apache.tomcat.unittest.TesterServletContext;
import org.apache.tomcat.util.descriptor.web.FilterDef;
import org.apache.tomcat.util.descriptor.web.FilterMap;

public class TestRateLimitFilter extends TomcatBaseTest {

    @Test
    public void TestRateLimitWith4Clients() throws Exception {

        int bucketRequests = 40;
        int bucketDuration = 4;

        FilterDef filterDef = new FilterDef();
        filterDef.addInitParameter(RateLimitFilter.PARAM_BUCKET_REQUESTS, String.valueOf(bucketRequests));
        filterDef.addInitParameter(RateLimitFilter.PARAM_BUCKET_DURATION, String.valueOf(bucketDuration));

        Tomcat tomcat = getTomcatInstance();
        Context root = tomcat.addContext("", TEMP_DIR);
        tomcat.start();

        MockFilterChain filterChain = new MockFilterChain();
        RateLimitFilter rateLimitFilter = testRateLimitFilter(filterDef, root);

        int allowedRequests = (int) Math.round(rateLimitFilter.bucketCounter.getRatio() * bucketRequests);

        long sleepTime = rateLimitFilter.bucketCounter.getMillisUntilNextBucket();
        System.out.printf("Sleeping %d millis for the next time bucket to start\n", sleepTime);
        Thread.sleep(sleepTime);

        TestClient tc1 = new TestClient(rateLimitFilter, filterChain, "10.20.20.5", 200, 5);
        TestClient tc2 = new TestClient(rateLimitFilter, filterChain, "10.20.20.10", 200, 10);

        TestClient tc3 = new TestClient(rateLimitFilter, filterChain, "10.20.20.20", 200, 20);
        TestClient tc4 = new TestClient(rateLimitFilter, filterChain, "10.20.20.40", 200, 40);

        Thread.sleep(5000);

        Assert.assertEquals(200, tc1.results[24]);    // only 25 requests made, all allowed

        Assert.assertEquals(200, tc2.results[49]);    // only 25 requests made, all allowed

        Assert.assertEquals(200, tc3.results[allowedRequests - 1]); // first allowedRequests allowed
        Assert.assertEquals(429, tc3.results[allowedRequests]);     // subsequent requests dropped

        Assert.assertEquals(200, tc4.results[allowedRequests - 1]); // first allowedRequests allowed
        Assert.assertEquals(429, tc4.results[allowedRequests]);     // subsequent requests dropped
    }

    private RateLimitFilter testRateLimitFilter(FilterDef filterDef, Context root)
            throws LifecycleException, IOException, ServletException {

        RateLimitFilter rateLimitFilter = new RateLimitFilter();
        filterDef.setFilterClass(RateLimitFilter.class.getName());
        filterDef.setFilter(rateLimitFilter);
        filterDef.setFilterName(RateLimitFilter.class.getName());
        root.addFilterDef(filterDef);

        FilterMap filterMap = new FilterMap();
        filterMap.setFilterName(RateLimitFilter.class.getName());
        filterMap.addURLPatternDecoded("*");
        root.addFilterMap(filterMap);

        FilterConfig filterConfig = generateFilterConfig(filterDef);

        rateLimitFilter.init(filterConfig);

        return rateLimitFilter;
        //*/
    }

    static class TestClient extends Thread {
        RateLimitFilter filter;
        FilterChain filterChain;
        String ip;

        int requests;
        int sleep;

        int[] results;

        TestClient(RateLimitFilter filter, FilterChain filterChain, String ip, int requests, int rps) {
            this.filter = filter;
            this.filterChain = filterChain;
            this.ip = ip;
            this.requests = requests;
            this.sleep = 1000 / rps;
            this.results = new int[requests];
            super.setDaemon(true);
            super.start();
        }

        @Override
        public void run() {
            try {
                for (int i = 0; i < requests; i++) {
                    MockHttpServletRequest request = new MockHttpServletRequest();
                    request.setRemoteAddr(ip);
                    TesterResponse response = new TesterResponseWithStatus();
                    response.setRequest(request);
                    filter.doFilter(request, response, filterChain);
                    results[i] = response.getStatus();
                    System.out.printf("%s %s: %s %d\n", ip, Instant.now(), i + 1, response.getStatus());
                    Thread.sleep(sleep);
                }
            }
            catch (Exception ex) {
                ex.printStackTrace();
            }
        }
    }

    static class TesterResponseWithStatus extends TesterResponse {

        int status = 200;
        String message = "OK";

        @Override
        public void sendError(int status, String message) throws IOException {
            this.status = status;
            this.message = message;
        }

        @Override
        public int getStatus() {
            return status;
        }
    }

    private static FilterConfig generateFilterConfig(FilterDef filterDef) {

        TesterServletContext mockServletContext = new TesterServletContext();
        Map<String, String> parameters = filterDef.getParameterMap();

        FilterConfig filterConfig = new FilterConfig() {

            @Override
            public String getFilterName() {
                return "rate-limit-filter";
            }

            @Override
            public ServletContext getServletContext() {
                return mockServletContext;
            }

            @Override
            public String getInitParameter(String name) {

                return parameters.get(name);
            }

            @Override
            public Enumeration<String> getInitParameterNames() {
                return null;
            }
        };

        return filterConfig;
    }

}
