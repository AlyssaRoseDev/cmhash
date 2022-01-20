# cmhash
A fast, non-cryptographic hash using Mersenne Primes for sharding and hash tables

# Algorithm
The basic algorithm is to xor the input with the state and multiply the input and a Mersenne Prime using a "widening" multiply and then storing the overflow as the next state. For the stateless function, the overflow is xor'd with the multiplied input instead.
