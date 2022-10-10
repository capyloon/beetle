#!/bin/bash

function setup_xcompile_envs() {
    # This is the Rust target architecture, which may not directly map to the clang triple.
    # TODO: do the inverse mapping instead since we'll get the clang arch when running
    #       from Android.mk
    export TARGET_ARCH=${TARGET_ARCH:-armv7-linux-androideabi}
    export ANDROID_API=${ANDROID_API:-33}
    export ANDROID_PLATFORM=${ANDROID_PLATFORM:-android-33}
    LIB_SUFFIX=""
    IS_MACOS=0

    case "$TARGET_ARCH" in
    armv7-linux-androideabi)
        TARGET_TRIPLE=armv7-linux-androideabi
	TARGET_INCLUDE=arm-linux-androideabi
        TOOLCHAIN_PREFIX=armv7a-linux-androideabi${ANDROID_API}
        ;;
    aarch64-linux-android)
        TARGET_TRIPLE=aarch64-linux-android
	TARGET_INCLUDE=${TARGET_TRIPLE}
        TOOLCHAIN_PREFIX=${TARGET_TRIPLE}${ANDROID_API}
        LIB_SUFFIX=64
        ;;
    x86_64-linux-android)
        TARGET_TRIPLE=x86_64-linux-android
        TARGET_INCLUDE=${TARGET_TRIPLE}
        TOOLCHAIN_PREFIX=${TARGET_TRIPLE}${ANDROID_API}
        LIB_SUFFIX=64
        ;;
    aarch64-apple-darwin)
        TARGET_TRIPLE=aarch64-apple-darwin
        TARGET_INCLUDE=${TARGET_TRIPLE}
        LIB_SUFFIX=64
	IS_MACOS=1
        ;;
    x86_64-apple-darwin)
        TARGET_TRIPLE=x86_64-apple-darwin
        TARGET_INCLUDE=${TARGET_TRIPLE}
        LIB_SUFFIX=64
	IS_MACOS=1
        ;;
    aarch64-unknown-linux-gnu)
        # Non-android targets will use the toolchain installed in $HOME/.mozbuild
        # since it's the same as the gecko one.
        TARGET_TRIPLE=aarch64-unknown-linux-gnu
        TARGET_INCLUDE=aarch64-linux-gnu
    esac

    HOST_OS=$(uname -s)

    if [ "$TARGET_ARCH" = "aarch64-apple-darwin" ]; then
        echo "Building for aarch64-apple-darwin"
        export SYSROOT=${OSX_CROSS}/MacOSX11.0.sdk/
        export SYS_INCLUDE_DIR=${SYSROOT}/usr/include
        export TOOLCHAIN_CC=${OSX_CROSS}/clang/bin/clang
        export TOOLCHAIN_CXX=${OSX_CROSS}/clang/bin/clang++
        export PATH=${OSX_CROSS}/cctools/bin:${OSX_CROSS}/clang/bin:${PATH}
        export LINKER=aarch64-apple-darwin-ld
        export LD=aarch64-apple-darwin-ld
    elif [ "$TARGET_ARCH" = "x86_64-apple-darwin" ]; then
        echo "Building for aarch64-apple-darwin"
        export SYSROOT=${OSX_CROSS}/MacOSX11.0.sdk/
        export SYS_INCLUDE_DIR=${SYSROOT}/usr/include
        export TOOLCHAIN_CC=${OSX_CROSS}/clang/bin/clang
        export TOOLCHAIN_CXX=${OSX_CROSS}/clang/bin/clang++
        export PATH=${OSX_CROSS}/cctools/bin:${OSX_CROSS}/clang/bin:${PATH}
        export LINKER=x86_64-apple-darwin-ld
        export LD=x86_64-apple-darwin-ld
    # Check that the BUILD_WITH_NDK_DIR environment variable is set
    # and build the .cargo/config file from it.
    elif [ -n "${BUILD_WITH_NDK_DIR}" ]; then
	if [ ! -d "${BUILD_WITH_NDK_DIR}" ]; then
            echo "${BUILD_WITH_NDK_DIR} doesn't exixt."
	    exit 1
	fi
        # If NDK_TOOLS_PATH is set and NULL, use the value NULL.
        NDK_TOOLS_PATH=${NDK_TOOLS_PATH-/toolchains/llvm/prebuilt/linux-x86_64}
        export TOOLCHAIN_CC=${TOOLCHAIN_PREFIX}-clang
        export TOOLCHAIN_CXX=${TOOLCHAIN_PREFIX}-clang++
        export SYSROOT=${BUILD_WITH_NDK_DIR}${NDK_TOOLS_PATH}/sysroot
        export SYS_INCLUDE_DIR=${SYSROOT}/usr/include
        export ANDROID_NDK=${BUILD_WITH_NDK_DIR}
        export PATH=${ANDROID_NDK}${NDK_TOOLS_PATH}/bin:${PATH}
        export AR=${BUILD_WITH_NDK_DIR}/toolchains/llvm/prebuilt/linux-x86_64/bin/llvm-ar
        export LINKER=${TOOLCHAIN_CC}

        echo "Building for ${TARGET_TRIPLE} using NDK '${BUILD_WITH_NDK_DIR}'"
    elif [ -n "${MOZBUILD}" ]; then
        export TOOLCHAIN_CC=${MOZBUILD}/clang/bin/clang
        export TOOLCHAIN_CXX=${MOZBUILD}/clang/bin/clang++
        export SYSROOT=${MOZBUILD}/sysroot-${TARGET_INCLUDE}
        export SYS_INCLUDE_DIR=${SYSROOT}/usr/include
        export PATH=${MOZBUILD}/clang/bin:${PATH}
        export AR=${BUILD_WITH_NDK_DIR}/toolchains/llvm/prebuilt/linux-x86_64/bin/llvm-ar
        export LINKER=${TOOLCHAIN_CC}

        echo "Building for ${TARGET_TRIPLE} using MOZBUILD '${MOZBUILD}'"
    else
        echo "Set BUILD_WITH_NDK_DIR to your ndk directory to build, or MOZBUILD for non-Android targets."
        exit 2
    fi

    XCFLAGS="-fPIC --sysroot=${SYSROOT} -I${SYS_INCLUDE_DIR} -I${SYS_INCLUDE_DIR}/${TARGET_INCLUDE}"
    CXXEXTRAS=""

    if [ "$IS_MACOS" = 1 ]; then
    	# Needed when cross-compiling rocksdb
    	export BINDGEN_EXTRA_CLANG_ARGS=${XCFLAGS}
    	export CRATE_CC_NO_DEFAULTS=1
    	# See https://gitanswer.net/error-thread-local-storage-is-not-supported-for-the-current-target-706365770
    	export MACOSX_DEPLOYMENT_TARGET=11.0
        CXXEXTRAS="-stdlib=libc++"
    fi

    if [ -n "${MOZBUILD}" ]; then
        export BINDGEN_EXTRA_CLANG_ARGS=${XCFLAGS}
	export CRATE_CC_NO_DEFAULTS=1
    fi
 
    export GIT_BUILD_INFO=$(
        git log -n 1 --pretty=format:"%H "
        date +%d/%m/%Y-%H:%M:%S
    )
}

function xcompile() {
    export CARGO_BUILD_TARGET=${TARGET_TRIPLE}
    export CARGO_CONFIG=$(pwd)/.cargo/config

    echo "Creating '$CARGO_CONFIG'"
    mkdir -p $(pwd)/.cargo
    cat <<EOF >$CARGO_CONFIG
[profile.release]
codegen-units = 1
debug = false
debug-assertions = false
lto = true
opt-level = 3
panic = "abort"
rpath = false

[target.${TARGET_TRIPLE}]
linker = "${LINKER}"
rustflags = [
  "-C", "opt-level=z",
EOF

    if [ "$TARGET_TRIPLE" = "aarch64-apple-darwin" ]; then
        cat <<EOF >>$CARGO_CONFIG
  "-C", "link-arg=-L${OSX_CROSS}/MacOSX11.0.sdk/usr/lib",
  "-C", "link-arg=-Z",
  "-C", "link-arg=-F${OSX_CROSS}/MacOSX11.0.sdk/System/Library/Frameworks/",
]
EOF
    elif [ "$TARGET_TRIPLE" = "x86_64-apple-darwin" ]; then
        cat <<EOF >>$CARGO_CONFIG
  "-C", "link-arg=-L${OSX_CROSS}/MacOSX11.0.sdk/usr/lib",
  "-C", "link-arg=-Z",
  "-C", "link-arg=-F${OSX_CROSS}/MacOSX11.0.sdk/System/Library/Frameworks/",
  "-C", "link-arg=${OSX_CROSS}/MacOSX11.0.sdk/usr/lib/crt1.o",
]
EOF
    elif [ "$TARGET_TRIPLE" = "aarch64-unknown-linux-gnu" ]; then
        cat <<EOF >>$CARGO_CONFIG
  "-C", "link-arg=-fuse-ld=lld",
  "-C", "link-arg=--target=${TARGET_TRIPLE}",
  "-C", "link-arg=--sysroot=${SYSROOT}",
  "-C", "link-arg=-L",
  "-C", "link-arg=${SYSROOT}/usr/lib",
  "-C", "link-arg=-L",
  "-C", "link-arg=${SYSROOT}/usr/lib/${TARGET_INCLUDE}",
  "-C", "link-arg=-v",
]
EOF
    else
        cat <<EOF >>$CARGO_CONFIG
  "-C", "link-arg=--sysroot=${SYSROOT}",
  "-C", "link-arg=-L",
  "-C", "link-arg=${GONK_DIR}/out/target/product/${GONK_PRODUCT}/system/lib${LIB_SUFFIX}",
  "-C", "link-arg=-L",
  "-C", "link-arg=${BUILD_WITH_NDK_DIR}/sysroot/usr/lib/${TARGET_TRIPLE}/${ANDROID_API}",
  "-C", "link-arg=-Wl,-rpath,${GONK_DIR}/out/target/product/${GONK_PRODUCT}/system/lib${LIB_SUFFIX}",
]
EOF
    fi

    # To add /usr/bin to $PATH, in order for host builds
    # of Rust crates to find 'cc' as a linker.
    # TODO: find a proper fix.
    export PATH=${PATH}:/usr/bin

    export CC=${TOOLCHAIN_CC}
    export CXX=${TOOLCHAIN_CXX}
    export LD=${LINKER}

    # And set CFLAGS again for the remaining crates.
    export CFLAGS="${XCFLAGS} --target=${TARGET_TRIPLE}"
    export CXXFLAGS="${XCFLAGS} ${CXXEXTRAS} --target=${TARGET_TRIPLE}"
 
    export TARGET_CC=${TOOLCHAIN_CC}
    export TARGET_LD=${TOOLCHAIN_CC}

#     cat <<EOF >$(pwd)/env.txt
# export CARGO_BUILD_TARGET=${TARGET_TRIPLE}
# export CARGO_CONFIG=$(pwd)/.cargo/config
# export CC=${TOOLCHAIN_CC}
# export CXX=${TOOLCHAIN_CXX}
# export LD=${TOOLCHAIN_CC}
# export CFLAGS=${XCFLAGS}
# export TARGET_CC=${TOOLCHAIN_CC}
# export TARGET_LD=${TOOLCHAIN_CC}
# EOF

    # printenv
    rustc --version
    cargo --version
    cargo build --target=${TARGET_TRIPLE} --features=${FEATURES} ${OPT}
}

function generate_breakpad_symbols() {
    if [ "$TARGET_ARCH" == "armv7-linux-androideabi" ]; then
        generate_breakpad_symbols_armv7 $1
    fi
}

function generate_breakpad_symbols_armv7() {
    # Generate symbols
    HOST_OS=$(uname -s)
    if [ "$HOST_OS" == "Darwin" ]; then
        DUMP_SYMS=../tools/dump_syms/dump_syms_mac
        return
    else
        DUMP_SYMS=../tools/dump_syms/dump_syms
    fi
    echo python ../tools/dump_syms/generate_breakpad_symbols.py --dump-syms-dir ../tools/dump_syms \
        --symbols-dir ../target/${TARGET_TRIPLE}/${BUILD_TYPE}/symbols --binary $1
    python ../tools/dump_syms/generate_breakpad_symbols.py --dump-syms-dir ../tools/dump_syms \
        --symbols-dir ../target/${TARGET_TRIPLE}/${BUILD_TYPE}/symbols --binary $1
}

function xstrip() {
    echo "Stripping with `which llvm-strip`"
    # Explicitely strip the binary since even release builds have symbols.
    llvm-strip $1
}
