FROM ghcr.io/bluebastion/bb-log-emitter:v0.3.8

ENTRYPOINT [ "/bin/bash", "-c", "$@" ]
CMD ["log_emitter"]
