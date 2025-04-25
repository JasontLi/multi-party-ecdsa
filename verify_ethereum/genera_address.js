const ethers = require("ethers");

const y_sum_s = [
    2,
    160,
    174,
    165,
    254,
    165,
    88,
    69,
    220,
    120,
    69,
    22,
    64,
    215,
    59,
    236,
    2,
    35,
    36,
    55,
    209,
    120,
    39,
    87,
    92,
    173,
    40,
    47,
    10,
    39,
    37,
    235,
    75
];

console.log(
    "address: ",
    ethers.utils.computeAddress("0x" + Buffer.from(y_sum_s).toString("hex"))
);