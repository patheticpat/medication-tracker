FROM nginx:alpine
ARG GIT_SHA=unknown
LABEL org.opencontainers.image.revision=$GIT_SHA
COPY dist /usr/share/nginx/html
COPY nginx.conf /etc/nginx/conf.d/default.conf
RUN echo "<!-- build: $GIT_SHA -->" >> /usr/share/nginx/html/index.html
