FROM node:16-alpine as build

WORKDIR /app
ENV PATH /app/node_modules/.bin:$PATH

# grab deps
COPY package.json ./
COPY package-lock.json ./

RUN npm install

RUN npm ci

# copy the  source
COPY . ./

RUN npm run build

# production environment
FROM nginx:stable-alpine

COPY nginx.conf /etc/nginx/conf.d/default.conf
COPY --from=build /app/build /usr/share/nginx/html

EXPOSE 80

CMD ["nginx", "-g", "daemon off;"]