#ifndef MAIN_H
#define MAIN_H

// GPIO PINS
#define LED_GREEN 0x04
#define LED_YELLOW 0x08


/* Initialized the software state machine according to the received cmd
 *
 * cmd: The command/register address received
 * */
void I2C_Slave_ProcessCMD(uint8_t cmd);
void initGPIO();
void initClockTo16MHz();
void initADC();

#endif
