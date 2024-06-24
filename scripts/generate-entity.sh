sudo docker stop postgres-migrations
sudo docker run -d --rm -P --network host -e POSTGRES_PASSWORD="1234" -e POSTGRES_USER="me" -e POSTGRES_DB="test" --name postgres-migrations postgres
sea-orm-cli migrate up --database-url postgresql://me:1234@127.0.0.1:5432/test
sea-orm-cli generate entity -o entity/src --database-url postgresql://me:1234@127.0.0.1:5432/test --lib --with-serde both
sudo docker stop postgres-migrations
