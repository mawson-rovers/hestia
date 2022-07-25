#ifndef I2C_H
#define I2C_H
#define SLAVE_ADDR  0x08
#define MAX_BUFFER_SIZE 2


union I2C_Packet_t{
 uint16_t sensor;
 uint8_t I2CPacket[sizeof(int)];
};

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
