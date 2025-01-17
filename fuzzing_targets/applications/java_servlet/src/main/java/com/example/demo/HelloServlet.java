package com.example.demo;

import java.io.*;
import java.util.stream.Collectors;

import com.alibaba.fastjson.JSONObject;

import javax.servlet.http.*;
import javax.servlet.annotation.*;

@WebServlet(name = "index", value = "/")
public class HelloServlet extends HttpServlet {
    public void doGet(HttpServletRequest request, HttpServletResponse response) throws IOException {
        response.setContentType("application/json");
        
        JSONObject result = new JSONObject();
        result.put("host", request.getHeader("Host"));
        result.put("content_length", request.getContentLength());
        result.put("transfer_encoding", request.getHeader("Transfer-Encoding"));
        String bodyContent = request.getReader().lines().collect(Collectors.joining(System.lineSeparator()));
        result.put("body_content", bodyContent);
        result.put("body_length", bodyContent.length());

        if (request.getHeader("X-Desync-Id") != null) {
            response.setHeader("X-Desync-Id", request.getHeader("X-Desync-Id"));
        }

        response.getWriter().println(result.toJSONString());
    }

    public void doPost(HttpServletRequest request, HttpServletResponse response) throws IOException {
        doGet(request, response);
    }

    public void destroy() {
    }
}
