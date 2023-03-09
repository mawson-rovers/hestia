#ifndef I2C_H
#define I2C_H
#define SLAVE_ADDR  0x08
#define MAX_BUFFER_SIZE 2


union I2C_Packet_t{
 uint16_t data;
 uint8_t I2CPacket[sizeof(uint16_t)];
};

static uint8_t TransmitLen = 0;

void CopyArray(const uint8_t *source);
void initI2C();
#endif
