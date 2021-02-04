FROM busybox
COPY target/release/cikit .
ENTRYPOINT [ "/cikit" ]