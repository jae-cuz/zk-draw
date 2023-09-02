import React, { useState } from "react";
import * as gacha from "../lib/wasm/gacha.js";

const keccak256 = require("keccak256");

const App = () => {
  const [hash, setHash] = useState("");
  const [randomNumber, setRandomNumber] = useState("");
  const [verificationResult, setVerificationResult] = useState("");
  const [preimage, setPreimage] = useState("");

  const loadWasm = async () => {
    await gacha.default();
    // 난수 생성 및 증명 생성
    const seed = BigInt(Math.floor(Math.random() * 1000));
    const params = gacha.setup_params(6);
    const randomNumber = gacha.generate_random(seed);
    const proof = gacha.proof_generate(seed, params);

    // 난수와 증명 표시
    setRandomNumber(String(randomNumber));
    console.log("Proof: ", proof);

    // 증명 검증
    const isVerified = gacha.proof_verify(params, randomNumber, proof);
    setVerificationResult(
      isVerified ? "Verification successful" : "Verification failed"
    );
  };

  const handlePreimageCheck = () => {
    // 입력된 preimage를 Keccak-256 해시화
    const hashedPreimage = keccak256(preimage).toString("hex");

    // 해시 값과 해시화된 preimage 비교
    if (hash === hashedPreimage) {
      if (BigInt(preimage) == BigInt(randomNumber)) {
        alert("Congratulations! You won.");
      } else {
        alert("Try agian..");
      }
    } else {
      alert("Your number doesn't match the hash");
    }
  };

  return (
    <div>
      <h1>ZK Draw</h1>
      <label>
        Enter a hash:
        <input
          type="text"
          value={hash}
          onChange={(e) => setHash(e.target.value)}
        />
      </label>
      <br />
      <button onClick={loadWasm}>Generate Random Number and Proof</button>
      <div>
        <p>Generated Random Number: {randomNumber}</p>
        <p>Verification Result: {verificationResult}</p>
      </div>
      <label>
        Enter a preimage for hash:
        <input
          type="text"
          value={preimage}
          onChange={(e) => setPreimage(e.target.value)}
        />
      </label>
      <br />
      <button onClick={handlePreimageCheck}>Check Preimage</button>
    </div>
  );
};

export default App;
