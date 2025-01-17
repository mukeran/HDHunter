#!/bin/sh

HTTPD_VER=2.4.57
APR_VER=1.7.4
APR_UTIL_VER=1.6.3
NGHTTP2_VER=1.55.1

echo 'Installing dependencies...'
sudo apt install -y libidn2-dev librtmp-dev libssh-dev libpsl-dev libgssapi-krb5-2 libkrb5-dev libk5crypto3 libcom-err2 libexpat1-dev

sudo apt install -y bzip2 libcurl4-openssl-dev libjansson-dev liblua5.2-dev libpcre3-dev libssl-dev libxml2-dev make wget zlib1g-dev uuid-dev libldap2-dev libbrotli-dev libzstd-dev brotli

if [ ! -d build ]; then
	mkdir build || exit 1
fi

echo 'Checking sources...'

ln -s "$(realpath $TARGETS_ROOT/runtime/c/hdhunter_api.h)" httpd-$HTTPD_VER/hdhunter_api.h

if [ ! -d apr-$APR_VER ]; then
    echo "Downloading apr-$APR_VER"
    curl -O https://archive.apache.org/dist/apr/apr-$APR_VER.tar.gz
    tar xzf apr-$APR_VER.tar.gz
    rm apr-$APR_VER.tar.gz
fi

if [ ! -d apr-util-$APR_UTIL_VER ]; then
    echo "Downloading apr-util-$APR_UTIL_VER"
    curl -O https://archive.apache.org/dist/apr/apr-util-$APR_UTIL_VER.tar.gz
    tar xzf apr-util-$APR_UTIL_VER.tar.gz
    rm apr-util-$APR_UTIL_VER.tar.gz
fi

if [ ! -d nghttp2-$NGHTTP2_VER ]; then
    echo "Downloading nghttp2-$NGHTTP2_VER"
    curl -L -O https://github.com/nghttp2/nghttp2/releases/download/v$NGHTTP2_VER/nghttp2-$NGHTTP2_VER.tar.gz
    tar xzf nghttp2-$NGHTTP2_VER.tar.gz
    rm nghttp2-$NGHTTP2_VER.tar.gz
fi

INSTALL_PREFIX="/hdhunter/apache"
DESTDIR="$(realpath "$PWD/build")"
NGHTTP2_PATH="$(realpath nghttp2-$NGHTTP2_VER)"
APR_PATH="$(realpath apr-$APR_VER)"
APR_UTIL_PATH="$(realpath apr-util-$APR_UTIL_VER)"
CFLAGS_SAN=" -fsanitize-coverage=trace-pc-guard -fPIC -g"
LIBS_HDHUNTER="$(realpath $TARGETS_ROOT/../target/debug/libhdhunter_rt.a)"
APACHE_MODULES=most

cd httpd-$HTTPD_VER

export CC="clang"
export CXX="clang++"

# If env APACHE_ONLY is set, only build apache
if [ -z "$APACHE_ONLY" ]; then
    echo "Compiling APR"
    cd "$APR_PATH"
    ./buildconf
    CFLAGS="$CFLAGS_SAN" ./configure --disable-shared --enable-static
    make clean
    make -j$(nproc)
    cd -

    echo "Compiling APR-UTIL"
    cd "$APR_UTIL_PATH"
    ./buildconf --with-apr="$APR_PATH"
    CFLAGS="$CFLAGS_SAN" ./configure --with-apr="$APR_PATH" --disable-shared --enable-static
    make clean
    make -j$(nproc)
    cd -

    echo "Compiling NGHTTP2"
    cd "$NGHTTP2_PATH"
    CFLAGS="$CFLAGS_SAN" CXXFLAGS="$CFLAGS_SAN" ./configure --disable-shared --enable-static
    make clean
    make -j$(nproc)
    cd -
fi

# If env MAKE_ONLY is set, only make apache
if [ -z "$MAKE_ONLY" ]; then
    echo "Install PATH: $INSTALL_PREFIX"
    ./buildconf --with-apr="$APR_PATH" --with-apr-util="$APR_UTIL_PATH"

    echo "Compiling HTTPD"
    CFLAGS="-I$NGHTTP2_PATH/lib/includes $CFLAGS_SAN -ggdb -O3" LDFLAGS="-L$NGHTTP2_PATH/lib" LIBS="-lpthread $LIBS_HDHUNTER" \
    ./configure \
            --prefix="$INSTALL_PREFIX" \
            --with-nghttp2="$NGHTTP2_PATH" \
            --enable-http2 \
            --enable-nghttp2-staticlib-deps \
            --with-mpm=event \
            --enable-unixd \
            --disable-pie \
            --disable-ssl \
            --enable-mods-static=$APACHE_MODULES \
            --with-apr="$APR_PATH" \
            --with-apr-util="$APR_UTIL_PATH"
fi

make clean
make
make DESTDIR=$DESTDIR install
