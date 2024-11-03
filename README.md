# Local Order Book
Maintain a local order book using the binance API.

# Example Usage

```
cargo run ETHBTC,BNBBTC,LTCBTC 2> error.log
```

The list of crypto pairs for which to maintain local order books is passed as a comma-separated 
list.

The program displays a live updating order book in the terminal. To cycle through the 
different crypto pairs, use the left and right arrow keys.

```
ETHBTC
        Bids          |         Asks
37.6587    0.03589    | 0.0359     16.3729
27.3363    0.03588    | 0.03591    35.4507
37.256     0.03587    | 0.03592    20.4893
40.8763    0.03586    | 0.03593    21.2693
24.3689    0.03585    | 0.03594    24.8523
22.3347    0.03584    | 0.03595    36.2908
35.7807    0.03583    | 0.03596    17.5179
41.2592    0.03582    | 0.03597    29.9266
81.5586    0.03581    | 0.03598    92.1528
31.8613    0.0358     | 0.03599    8.8891
70.503     0.03579    | 0.036      157.6393
3.29       0.03578    | 0.03601    15.4723
0.6316     0.03577    | 0.03602    94.787
2.922      0.03576    | 0.03603    2.8373
11.7124    0.03575    | 0.03604    0.6729
59.4768    0.03574    | 0.03605    1.0411
0.8339     0.03573    | 0.03606    23.236
70.9453    0.03572    | 0.03607    71.7725
0.8587     0.03571    | 0.03608    0.7511
23.0898    0.0357     | 0.03609    24.5028
```

The program can be terminated by pressing `ESC`

# Logs 
Logs are written to stderr. To not disrupt the terminal output, it is best to redirect stderr to
a file.


