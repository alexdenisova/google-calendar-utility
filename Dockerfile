ARG VERSION="3.16"

FROM alpine:${VERSION} AS final

LABEL org.opencontainers.image.source = "https://github.com/alexdenisova/google-calendar-utility"

RUN \
    apk --no-cache add curl jq \
    && rm -rf /var/cache/apk/* /tmp/*

COPY dist/bin/* /bin/
RUN chmod +x /bin/*

EXPOSE 8080/tcp
WORKDIR /workspace
ENTRYPOINT [""]
