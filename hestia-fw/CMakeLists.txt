cmake_minimum_required(VERSION 3.5)
project(hestia-fw)

set(SOURCE_FILES main.c i2c.c)

add_platform_executable(hestia "" ${SOURCE_FILES})
