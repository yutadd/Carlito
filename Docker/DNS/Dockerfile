FROM ubuntu/bind9
COPY ./named.conf /etc/bind/named.conf
COPY ./yutadd.com.zone /etc/bind/
RUN apt-get update && apt-get install -y dnsutils