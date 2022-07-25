#include <msp430.h>
#include <stdint.h>
#include <stdbool.h>
#include "main.h"
#include "i2c.h"

uint8_t TransmitBuffer[MAX_BUFFER_SIZE] = {0};
uint8_t TransmitIndex = 0;
volatile int tmp = 0;

void CopyArray(uint8_t *source)
{
    // copy data into transmitt buffer
    //TODO disable interupt
    //TODO take sensor message
    uint8_t copyIndex = 0;
    for (copyIndex = 0; copyIndex < MAX_BUFFER_SIZE; copyIndex++)
    {
        TransmitBuffer[copyIndex] = source[copyIndex];
    }
    TransmitIndex = 0;
}


//******************************************************************************
// Device Initialization *******************************************************
//******************************************************************************


void initI2C()
{
    UCB0CTL1 |= UCSWRST;                      // Enable SW reset
    UCB0CTL0 = UCMODE_3 + UCSYNC;             // I2C Slave, synchronous mode
    UCB0I2COA = SLAVE_ADDR;                   // Own Address
    UCB0CTL1 &= ~UCSWRST;                     // Clear SW reset, resume operation
    UCB0I2CIE |= UCSTPIE + UCSTTIE;           // Enable STT and STP interrupt
    IE2 |= UCB0RXIE | UCB0TXIE;               // Enable TX, RX interrupt
}


//******************************************************************************
// I2C Interrupt For Received and Transmitted Data******************************
//******************************************************************************

#if defined(__TI_COMPILER_VERSION__) || defined(__IAR_SYSTEMS_ICC__)
#pragma vector = USCIAB0TX_VECTOR
__interrupt void USCIAB0TX_ISR(void)
#elif defined(__GNUC__)
void __attribute__ ((interrupt(USCIAB0TX_VECTOR))) USCIAB0TX_ISR (void)
#else
#error Compiler not supported!
#endif
{
  // UCB0IV;
  if (IFG2 & UCB0RXIFG)                 // Receive Data Interrupt
  {
      //Must read from UCB0RXBUF
      uint8_t rx_val = UCB0RXBUF;
//      P5OUT |= LED_GREEN;                          // LED_1 on
//      P5OUT &= ~LED_YELLOW;                         // LED_2 off
      I2C_Slave_ProcessCMD(rx_val);
  }
  else if (IFG2 & UCB0TXIFG)            // Transmit Data Interrupt
  {
//      P5OUT |= LED_YELLOW;                          // LED_2 on
//      P5OUT &= ~LED_GREEN;                         // LED_1 off
      //Must write to UCB0TXBUF
      if(TransmitIndex < MAX_BUFFER_SIZE){
          UCB0TXBUF = TransmitBuffer[TransmitIndex++];
      }else{
          UCB0TXBUF = 0; // Out of range
      }
  }
}



//******************************************************************************
// I2C Interrupt For Start, Restart, Nack, Stop ********************************
//******************************************************************************

#if defined(__TI_COMPILER_VERSION__) || defined(__IAR_SYSTEMS_ICC__)
#pragma vector = USCIAB0RX_VECTOR
__interrupt void USCIAB0RX_ISR(void)
#elif defined(__GNUC__)
void __attribute__ ((interrupt(USCIAB0RX_VECTOR))) USCIAB0RX_ISR (void)
#else
#error Compiler not supported!
#endif
{
    if (UCB0STAT & UCSTPIFG)                        //Stop or NACK Interrupt
    {
        if(UCB0STAT){
            TransmitIndex = 0;
        }
        UCB0STAT &=
            ~(UCSTTIFG + UCSTPIFG + UCNACKIFG);     //Clear START/STOP/NACK Flags
    }
    if (UCB0STAT & UCSTTIFG)
    {
        UCB0STAT &= ~(UCSTTIFG);                    //Clear START Flags
    }
}
