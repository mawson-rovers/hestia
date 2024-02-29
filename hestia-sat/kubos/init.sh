#!/bin/bash 

echo 45 > /sys/class/gpio/export
echo 27 > /sys/class/gpio/export   
echo 47 > /sys/class/gpio/export   
echo 68 > /sys/class/gpio/export
echo 69 > /sys/class/gpio/export
echo 44 > /sys/class/gpio/export
echo 46 > /sys/class/gpio/export
echo 26 > /sys/class/gpio/export
echo 65 > /sys/class/gpio/export

echo 49 > /sys/class/gpio/export
echo 115 > /sys/class/gpio/export
echo 60 > /sys/class/gpio/export
echo 48 > /sys/class/gpio/export
echo 117 > /sys/class/gpio/export
echo 112 > /sys/class/gpio/export

echo out > /sys/class/gpio/gpio45/direction
echo out > /sys/class/gpio/gpio27/direction
echo out > /sys/class/gpio/gpio47/direction

echo in > /sys/class/gpio/gpio68/direction
echo in > /sys/class/gpio/gpio69/direction
echo out > /sys/class/gpio/gpio115/direction
echo out > /sys/class/gpio/gpio117/direction

echo in > /sys/class/gpio/gpio60/direction 
echo in > /sys/class/gpio/gpio48/direction
echo out > /sys/class/gpio/gpio44/direction
echo out > /sys/class/gpio/gpio26/direction

echo in > /sys/class/gpio/gpio49/direction
echo in > /sys/class/gpio/gpio112/direction

