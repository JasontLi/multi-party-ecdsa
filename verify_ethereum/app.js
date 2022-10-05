const ethers = require("ethers");

(async () => {
    const convertHashToHex = (value) => {
        return value.map(v => v.toString(16).padStart(2, '0')).join('');
    } 

    const r1 = "0x" + convertHashToHex([108,170,90,166,97,48,215,31,135,180,132,51,79,217,196,169,213,135,203,208,78,48,162,190,142,184,201,38,87,103,129,9])
    const s1 = "0x" + convertHashToHex([69,147,11,8,79,243,249,250,90,244,221,118,91,162,161,77,9,183,108,74,138,39,215,103,97,179,188,255,176,23,116,33])
    const recid1 = 1
    
    const expandedSig1 = {
        r: r1,
        s: s1,
        v: recid1
    }

    const recovered1 = ethers.utils.recoverAddress(ethers.utils.keccak256(ethers.utils.toUtf8Bytes("hello")), expandedSig1)
    console.log(recovered1);

    const r2 = "0x" + convertHashToHex([254,213,200,231,129,112,233,91,53,135,77,210,50,225,53,52,27,65,119,184,46,154,13,233,167,110,89,179,190,3,236,37])
    const s2 = "0x" + convertHashToHex([35,39,225,77,72,110,155,1,42,133,135,158,241,23,247,248,146,181,201,175,93,250,6,127,93,90,65,58,71,176,139,14])
    const recid2 = 1
    
    const expandedSig2 = {
        r: r2,
        s: s2,
        v: recid2
    }

    const recovered2 = ethers.utils.recoverAddress(ethers.utils.keccak256(ethers.utils.toUtf8Bytes("hello")), expandedSig2)
    console.log(recovered2)
})()
