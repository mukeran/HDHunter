# Build HDHunter from scratch

## **Step 1**: Download source from GitHub or Zenodo

**From GitHub**:
```shell
git clone https://github.com/mukeran/HDHunter && cd HDHunter
```

**From Zenodo**:

Download and extract source code from https://zenodo.org/records/14557764.

## **Step 2**: Compile and install language interpreters

Make sure the build essentials are installed before compiling.

```shell
# Install build-essential
sudo apt update
sudo apt install build-essential

# Install dependencies for Python and Ruby
sudo apt install zlib1g-dev libbz2-dev libreadline-dev libssl-dev libsqlite3-dev libyaml-dev

sudo apt install autoconf bison
```

HDHunter uses LLVM's SanitierCoverage for C and C++ projects. Use following code to install llvm/clang.

```shell
sudo apt install clang
```

Now, let's build language interpreters.

```shell
# Build modified Witcher-java (please have docker installed in advance)
cd vendors/Witcher-java && sh ./build.sh && cd -

# Build modified Witcher-python
cd vendors/Witcher-python && sh ./build.sh && cd -

# Build modified ruby3 version of Witcher-ruby
cd vendors/hdhunter-ruby3 && sh ./build.sh && cd -
```

## **Step 3** *(Optional)*: Build language bindings for Java, Ruby and CGI applications for Java

```shell
# Make sure that the official JDK and Maven are properly installed
cd fuzzing_targets/runtime/java/hdhunter-api-java && mvn clean compile jar:jar && cd -
cd fuzzing_targets/applications/java_servlet && mvn clean compile war:war && cd -

# Make sure that the official Ruby is properly installed
cd fuzzing_targets/runtime/ruby/hdhunter-api-ruby && gem build hdhunter-api-ruby.gemspec && cd -
```

## **Step 4**: Build HDHunter

Make sure Rust and cargo-make has installed.

```shell
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install cargo-make
cargo install cargo-make
```

Build HDHunter.

```shell
cargo make
```

## **Step 5**: Build fuzzing targets

Make sure pax-utils is installed (lddtree is needed).

```shell
sudo apt install pax-utils
```

Take Apache HTTPd for example:

```shell
cd fuzzing_targets && mkdir -p dist && make apache && cd -
```

The packed output will be stored in `fuzzing_targets/dist/`.