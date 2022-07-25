//******************************************************************************
//   MSP430x261x Demo - USCI_B0, I2C Slave multiple byte TX/RX
//
//   Description: I2C master communicates to I2C slave sending and receiving
//   3 different messages of different length. (This is the slave code). The
//   slave will be in LPM0 mode, waiting for the master to initiate the
//   communication. The slave will send/receive bytes based on the master's
//   request. The slave will handle I2C bytes sent/received using the
//   I2C interrupt.
//   ACLK = NA, MCLK = SMCLK = DCO 16MHz.
//
//
//                   MSP430F2619         3.3V
//                 -----------------   /|\ /|\
//            /|\ |                 |   |  4.7k
//             |  |                 |  4.7k |
//             ---|RST              |   |   |
//                |                 |   |   |
//                |             P3.2|---|---+- I2C Clock (UCB0SCL)
//                |                 |   |
//                |             P3.1|---+----- I2C Data (UCB0SDA)
//                |                 |
//                |                 |
//
//   Nima Eskandari and Ryan Meredith
//   Texas Instruments Inc.
//   January 2018
//   Built with CCS V7.3
//******************************************************************************

#include <msp430.h> 
#include <stdint.h>
#include <stdbool.h>
#include "main.h"
#include "i2c.h"

union I2C_Packet_t message;

//******************************************************************************
// Main ************************************************************************
// Enters LPM0 and waits for I2C interrupts. The data sent from the master is  *
// then interpreted and the device will respond accordingly                    *
//******************************************************************************


int main(void) {
    WDTCTL = WDTPW | WDTHOLD;   // Stop watchdog timer
    message.sensor = 100;

    initClockTo16MHz();
    initGPIO();
    initI2C();
    initADC();

//    __bis_SR_register(LPM0_bits + GIE);
    for (;;)
    {
      ADC12CTL0 |= ADC12SC;                   // Start convn, software controlled
      __bis_SR_register(CPUOFF + GIE + LPM0_bits);        // LPM0, ADC12_ISR will force exit
    }
    return 0;
}

void I2C_Slave_ProcessCMD(uint8_t cmd)
{
    //Fill out the TransmitBuffer
    CopyArray(message.I2CPacket);
}

void initClockTo16MHz()
{
    if (CALBC1_16MHZ==0xFF)                  // If calibration constant erased
    {
        while(1);                               // do not load, trap CPU!!
    }
    DCOCTL = 0;                               // Select lowest DCOx and MODx settings
    BCSCTL1 = CALBC1_16MHZ;                    // Set DCO
    DCOCTL = CALDCO_16MHZ;
}

void initGPIO()
{
    //I2C Pins
    P3SEL |= BIT1 | BIT2;                     // P3.1,2 option select
    P5DIR |= 0x0F;                            // Set P1.0 to output direction
    P5OUT &= ~(LED_YELLOW | LED_GREEN);                          // Turn off red led
}

void initADC()
{
    ADC12CTL0 = SHT0_2 + ADC12ON;             // Set sampling time, turn on ADC12
    ADC12CTL1 = SHP;                          // Use sampling timer
    ADC12IE = 0x01;                           // Enable interrupt
    ADC12CTL0 |= ENC;                         // Conversion enabled
    P6DIR &= ~0x01;                            // P6.0, i/p
    P6SEL |= 0x01;                            // P6.0-ADC option select
}

// ADC12 interrupt service routine
#if defined(__TI_COMPILER_VERSION__) || defined(__IAR_SYSTEMS_ICC__)
#pragma vector=ADC12_VECTOR
__interrupt void ADC12_ISR (void)
#elif defined(__GNUC__)
void __attribute__ ((interrupt(ADC12_VECTOR))) ADC12_ISR (void)
#else
#error Compiler not supported!
#endif
{
    P5OUT |= LED_GREEN;                          // LED_1 on
    P5OUT &= ~LED_YELLOW;                         // LED_2 off
    message.sensor = ADC12MEM0;
    __bic_SR_register_on_exit(CPUOFF);      // Clear CPUOFF bit from 0(SR)
}
