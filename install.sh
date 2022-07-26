#!/bin/sh

set -x
set -e

cargo build --release

sudo rm -f  /usr/local/bin/wedding-save-the-date || true
sudo rm -rf /var/www/wedding-save-the-date/static

sudo cp wedding-save-the-date.service        /etc/systemd/system/wedding-save-the-date.service
sudo cp -r static                            /var/www/wedding-save-the-date/.
sudo cp target/release/wedding-save-the-date /usr/local/bin/wedding-save-the-date
sudo cp syslog-wedding-save-the-date.conf    /etc/rsyslog.d/syslog-wedding-save-the-date.conf

sudo touch            /var/log/wedding-save-the-date.log
sudo chown syslog:adm /var/log/wedding-save-the-date.log

sudo systemctl daemon-reload

sudo systemctl restart rsyslog.service

sudo systemctl status  wedding-save-the-date.service --no-pager || true
sudo systemctl enable  wedding-save-the-date.service
sudo systemctl restart wedding-save-the-date.service
