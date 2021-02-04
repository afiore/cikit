FROM busybox
COPY target/x86_64-unknown-linux-musl/release/cikit .
RUN ["chmod", "+x", "/cikit"]
ENTRYPOINT [ "/cikit" ]
