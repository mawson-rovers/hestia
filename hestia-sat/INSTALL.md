Use the `upload.sh` script to copy everything from your local machine to the Beaglebone:

```shell
$ cd ~/src/mawson/hestia/hestia-sat
$ cargo build --release --bin uts-web --bin uts-cli --bin uts-log
$ ./upload.sh target/arm-unknown-linux-gnueabihf/release/uts-web
```

Then SSH into the Beaglebone and configure everything.

### Systemd services

Install the systemd services and paths using systemctl on the Beaglebone:

```shell
debian@beaglebone:~$ cd ~/uts
debian@beaglebone:~/uts$ sudo systemctl link systemd/*
Created symlink /etc/systemd/system/i2c-pin-config.service → /home/debian/uts/systemd/i2c-pin-config.service.
Created symlink /etc/systemd/system/uts-log-monitor.path → /home/debian/uts/systemd/uts-log-monitor.path.
Created symlink /etc/systemd/system/uts-log-monitor.service → /home/debian/uts/systemd/uts-log-monitor.service.
Created symlink /etc/systemd/system/uts-log.service → /home/debian/uts/systemd/uts-log.service.
Created symlink /etc/systemd/system/uts-web-monitor.path → /home/debian/uts/systemd/uts-web-monitor.path.
Created symlink /etc/systemd/system/uts-web-monitor.service → /home/debian/uts/systemd/uts-web-monitor.service.
Created symlink /etc/systemd/system/uts-web-nginx.path → /home/debian/uts/systemd/uts-web-nginx.path.
Created symlink /etc/systemd/system/uts-web-nginx.service → /home/debian/uts/systemd/uts-web-nginx.service.
Created symlink /etc/systemd/system/uts-web.service → /home/debian/uts/systemd/uts-web.service.

debian@beaglebone:~/uts$ cd systemd/

debian@beaglebone:~/uts/systemd$ ls
i2c-pin-config.service  uts-log-monitor.service  uts-web-monitor.path     uts-web-nginx.path     uts-web.service
uts-log-monitor.path    uts-log.service          uts-web-monitor.service  uts-web-nginx.service

debian@beaglebone:~/uts/systemd$ sudo systemctl enable *
Created symlink /etc/systemd/system/multi-user.target.wants/i2c-pin-config.service → /home/debian/uts/systemd/i2c-pin-config.service.
Created symlink /etc/systemd/system/multi-user.target.wants/uts-log-monitor.path → /home/debian/uts/systemd/uts-log-monitor.path.
Created symlink /etc/systemd/system/multi-user.target.wants/uts-log-monitor.service → /home/debian/uts/systemd/uts-log-monitor.service.
Created symlink /etc/systemd/system/multi-user.target.wants/uts-log.service → /home/debian/uts/systemd/uts-log.service.
Created symlink /etc/systemd/system/multi-user.target.wants/uts-web-monitor.path → /home/debian/uts/systemd/uts-web-monitor.path.
Created symlink /etc/systemd/system/multi-user.target.wants/uts-web-monitor.service → /home/debian/uts/systemd/uts-web-monitor.service.
Created symlink /etc/systemd/system/multi-user.target.wants/uts-web-nginx.path → /home/debian/uts/systemd/uts-web-nginx.path.
Created symlink /etc/systemd/system/multi-user.target.wants/uts-web-nginx.service → /home/debian/uts/systemd/uts-web-nginx.service.
Created symlink /etc/systemd/system/multi-user.target.wants/uts-web.service → /home/debian/uts/systemd/uts-web.service.

debian@beaglebone:~/uts/systemd$ sudo systemctl start i2c-pin-config uts-log uts-web
```

### Nginx config

Then link in the nginx config:

```shell
debian@b2:~$ cd /etc/nginx/sites-available/

debian@b2:/etc/nginx/sites-available$ sudo ln -s ~/uts/nginx/uts-web-nginx.conf .

debian@b2:/etc/nginx/sites-available$ cd ../sites-enabled/

debian@b2:/etc/nginx/sites-enabled$ sudo ln -s /etc/nginx/sites-available/uts-web-nginx.conf .

debian@b2:/etc/nginx/sites-enabled$ ls -l
total 0
lrwxrwxrwx 1 root root 34 Apr  6  2020 default -> /etc/nginx/sites-available/default
lrwxrwxrwx 1 root root 45 Jul  3 17:08 uts-web-nginx.conf -> /etc/nginx/sites-available/uts-web-nginx.conf

debian@b2:/etc/nginx/sites-enabled$ sudo systemctl reload nginx
```

### Link binaries into ~/bin

Next, link the uts binaries into `~/bin` so you can run them at the command line.

```shell
debian@b2:~$ cd ~/bin
debian@b2:~$ ln -s ~/uts/bin/* .
```

### Reboot

Restart the Beaglebone to make sure everything has started properly:

```shell
debian@beaglebone:~/uts/systemd$ sudo reboot now
```

Once the system has restarted, you should be able to access the web interface on
[http://beaglebone.local:5000](http://beaglebone.local:5000).

Log files are stored in `/home/debian/uts/logs`.
