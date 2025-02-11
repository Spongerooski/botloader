FROM node:16-alpine as build-docs

WORKDIR /app/components/runtime/docgen
ENV PATH /app/node_modules/.bin:$PATH

# grab deps
COPY components/runtime/docgen/package.json ./
COPY components/runtime/docgen/package-lock.json ./

RUN npm ci

# copy the  source
COPY components/runtime/ ../

RUN npm run build

FROM node:16-alpine as build-main

# Prepare frontend-common
WORKDIR /app/frontend-common

COPY frontend-common/package.json ./
COPY frontend-common/package-lock.json ./

RUN npm ci

COPY frontend-common/ ./

# Build frontend
WORKDIR /app/frontend

COPY frontend/package.json ./
COPY frontend/package-lock.json ./

RUN npm ci

COPY frontend/ ./
COPY --from=build-docs /app/components/runtime/docgen/docs public/docs

# Build config
ARG BOTLOADER_API_BASE
ENV REACT_APP_BOTLOADER_API_BASE=$BOTLOADER_API_BASE

ARG BOTLOADER_CLIENT_ID
ENV REACT_APP_BOTLOADER_CLIENT_ID=$BOTLOADER_CLIENT_ID

RUN npm run build

# production environment
FROM nginx:stable-alpine

COPY frontend/nginx.conf /etc/nginx/conf.d/default.conf
COPY --from=build-main /app/frontend/build /usr/share/nginx/html

EXPOSE 80

ENTRYPOINT ["nginx", "-g", "daemon off;"]