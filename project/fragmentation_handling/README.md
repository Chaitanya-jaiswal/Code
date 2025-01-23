# Fragmentaiton & Assembler
This library provides the user 
with functionalities to fragment a high
level message and it's reconstruction

## Fragmentation
The first bit of data rapresented by a u8
is a recognition bit, when the serialize 
function is called this bit is will form
the data of the first fragment

## Assembler
Takes a vector of fragments, checks its wholeness,
meaning that all the fragments are present or not,
if they are it reconstruct the message recognizing 
its type by the first fragment (the one with the 
"recognition bit").
The wholeness of the fragments is checked by checksum 
of the size and fragment index.

## Message Types
The high-level messages are of types:
### -String
### -DynamicImage
### -AudioSource
### -DefaultsRequest
### -ContentRequest
### -ChatRequest
