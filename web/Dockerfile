FROM nginx:alpine
ARG GIT_SHA=unknown
LABEL org.opencontainers.image.revision=$GIT_SHA
COPY dist /usr/share/nginx/html
RUN echo "{\"sha\": \"$GIT_SHA\"}" > /usr/share/nginx/html/version.json
COPY nginx.conf /etc/nginx/conf.d/default.conf
