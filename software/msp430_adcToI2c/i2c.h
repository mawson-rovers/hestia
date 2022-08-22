#ifndef I2C_H
#define I2C_H
#define SLAVE_ADDR  0x08
#define MAX_BUFFER_SIZE 2

// ic2 commands
// #TODO think there was some example code that did this better but anyway
#define COMMAND_SENSOR_LOW       0x01
#define COMMAND_SENSOR_HIGH      0x0B
#define COMMAND_HEATER_MODE      0x40
#define COMMAND_TARGET_TEMP      0x41
#define COMMAND_TARGET_SENSOR    0x42
#define COMMAND_PWM_FREQUENCY    0x43
#define COMMAND_RESET            0x50

union I2C_Packet_t{
 uint16_t data;
 uint8_t I2CPacket[sizeof(int)];
};

static uint8_t TransmitLen = 0;

/* The transaction between the slave and master is completed. Uses cmd
 * to do post transaction operations. (Place data from ReceiveBuffer
 * to the corresponding buffer based in the last received cmd)
 *
 * cmd: The command/register address corresponding to the completed
 * transaction
 */
void I2C_Slave_TransactionDone(uint8_t cmd);
void CopyArray(uint8_t *source);
void initI2C();
#endif
