docker run --user root --rm --name srtool -e PACKAGE=gpu-runtime -e BUILD_OPTS= -e DEFAULT_FEATURES= -e PROFILE=production -v /home/$USER/project/gpu:/build -v /tmp/cargo:/cargo-home paritytech/srtool:1.70.0-0.11.0 build
