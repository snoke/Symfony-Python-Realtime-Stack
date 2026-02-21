FROM php:8.4-cli
ARG INSTALL_GRPC=0
WORKDIR /app
RUN apt-get update && apt-get install -y \
    git unzip pkg-config \
    && if [ "$INSTALL_GRPC" = "1" ]; then apt-get install -y libssl-dev zlib1g-dev; fi \
    && rm -rf /var/lib/apt/lists/*

RUN if [ "$INSTALL_GRPC" = "1" ]; then pecl install grpc && docker-php-ext-enable grpc; fi \
    && docker-php-ext-install sockets

COPY --from=composer:2 /usr/bin/composer /usr/bin/composer
COPY symfony /app
RUN if [ "$INSTALL_GRPC" = "1" ]; then \
      composer install --no-interaction --prefer-dist; \
    else \
      composer install --no-interaction --prefer-dist --ignore-platform-req=ext-grpc; \
    fi
EXPOSE 8000
CMD ["php", "-S", "0.0.0.0:8000", "-t", "public", "public/index.php"]
