/**
 * Program IDL in camelCase format in order to be used in JS/TS.
 *
 * Note that this is only a type helper and is not the actual IDL. The original
 * IDL can be found at `target/idl/dloom_locker.json`.
 */
export type DloomLocker = {
  "address": "AVfmdPiqXfc15Pt8PPRXxTP5oMs4D1CdijARiz8mFMFD",
  "metadata": {
    "name": "dloomLocker",
    "version": "0.1.0",
    "spec": "0.1.0",
    "description": "Created with Anchor"
  },
  "instructions": [
    {
      "name": "handleBurnTokens",
      "discriminator": [
        143,
        14,
        45,
        1,
        53,
        119,
        66,
        91
      ],
      "accounts": [
        {
          "name": "burner",
          "writable": true,
          "signer": true
        },
        {
          "name": "tokenMint",
          "writable": true
        },
        {
          "name": "userTokenAccount",
          "writable": true
        },
        {
          "name": "tokenProgram"
        }
      ],
      "args": [
        {
          "name": "amount",
          "type": "u64"
        }
      ]
    },
    {
      "name": "handleCloseVault",
      "discriminator": [
        118,
        174,
        166,
        116,
        227,
        61,
        190,
        64
      ],
      "accounts": [
        {
          "name": "owner",
          "writable": true,
          "signer": true,
          "relations": [
            "lockRecord"
          ]
        },
        {
          "name": "lockRecord",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  108,
                  111,
                  99,
                  107,
                  95,
                  114,
                  101,
                  99,
                  111,
                  114,
                  100
                ]
              },
              {
                "kind": "account",
                "path": "owner"
              },
              {
                "kind": "account",
                "path": "tokenMint"
              },
              {
                "kind": "arg",
                "path": "lockId"
              }
            ]
          }
        },
        {
          "name": "vault",
          "writable": true
        },
        {
          "name": "tokenMint"
        },
        {
          "name": "tokenProgram"
        }
      ],
      "args": [
        {
          "name": "lockId",
          "type": "u64"
        }
      ]
    },
    {
      "name": "handleLockTokens",
      "discriminator": [
        34,
        232,
        189,
        9,
        245,
        169,
        129,
        96
      ],
      "accounts": [
        {
          "name": "owner",
          "writable": true,
          "signer": true
        },
        {
          "name": "tokenMint"
        },
        {
          "name": "lockRecord",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  108,
                  111,
                  99,
                  107,
                  95,
                  114,
                  101,
                  99,
                  111,
                  114,
                  100
                ]
              },
              {
                "kind": "account",
                "path": "owner"
              },
              {
                "kind": "account",
                "path": "tokenMint"
              },
              {
                "kind": "arg",
                "path": "lockId"
              }
            ]
          }
        },
        {
          "name": "vault",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  118,
                  97,
                  117,
                  108,
                  116
                ]
              },
              {
                "kind": "account",
                "path": "lockRecord"
              }
            ]
          }
        },
        {
          "name": "userTokenAccount",
          "writable": true
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        },
        {
          "name": "tokenProgram"
        },
        {
          "name": "rent",
          "address": "SysvarRent111111111111111111111111111111111"
        }
      ],
      "args": [
        {
          "name": "amount",
          "type": "u64"
        },
        {
          "name": "unlockTimestamp",
          "type": "i64"
        },
        {
          "name": "lockId",
          "type": "u64"
        }
      ]
    },
    {
      "name": "handleWithdrawTokens",
      "discriminator": [
        230,
        69,
        126,
        254,
        23,
        189,
        147,
        215
      ],
      "accounts": [
        {
          "name": "owner",
          "writable": true,
          "signer": true,
          "relations": [
            "lockRecord"
          ]
        },
        {
          "name": "lockRecord",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  108,
                  111,
                  99,
                  107,
                  95,
                  114,
                  101,
                  99,
                  111,
                  114,
                  100
                ]
              },
              {
                "kind": "account",
                "path": "owner"
              },
              {
                "kind": "account",
                "path": "tokenMint"
              },
              {
                "kind": "arg",
                "path": "lockId"
              }
            ]
          }
        },
        {
          "name": "vault",
          "writable": true
        },
        {
          "name": "userTokenAccount",
          "writable": true
        },
        {
          "name": "tokenMint"
        },
        {
          "name": "tokenProgram"
        }
      ],
      "args": [
        {
          "name": "lockId",
          "type": "u64"
        }
      ]
    }
  ],
  "accounts": [
    {
      "name": "lockRecord",
      "discriminator": [
        157,
        145,
        17,
        26,
        171,
        35,
        61,
        131
      ]
    }
  ],
  "events": [
    {
      "name": "tokensBurned",
      "discriminator": [
        230,
        255,
        34,
        113,
        226,
        53,
        227,
        9
      ]
    },
    {
      "name": "tokensLocked",
      "discriminator": [
        63,
        184,
        201,
        20,
        203,
        194,
        249,
        138
      ]
    },
    {
      "name": "tokensWithdrawn",
      "discriminator": [
        30,
        116,
        110,
        147,
        87,
        89,
        9,
        158
      ]
    }
  ],
  "errors": [
    {
      "code": 6000,
      "name": "unlockDateInPast",
      "msg": "The unlock date must be in the future."
    },
    {
      "code": 6001,
      "name": "zeroAmount",
      "msg": "The amount to lock or burn must be greater than zero."
    },
    {
      "code": 6002,
      "name": "stillLocked",
      "msg": "These tokens are still locked."
    },
    {
      "code": 6003,
      "name": "lockDurationTooLong",
      "msg": "The lock duration is too long."
    }
  ],
  "types": [
    {
      "name": "lockRecord",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "bump",
            "type": "u8"
          },
          {
            "name": "owner",
            "type": "pubkey"
          },
          {
            "name": "mint",
            "type": "pubkey"
          },
          {
            "name": "vault",
            "type": "pubkey"
          },
          {
            "name": "amount",
            "type": "u64"
          },
          {
            "name": "unlockTimestamp",
            "type": "i64"
          },
          {
            "name": "id",
            "type": "u64"
          }
        ]
      }
    },
    {
      "name": "tokensBurned",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "burner",
            "type": "pubkey"
          },
          {
            "name": "mint",
            "type": "pubkey"
          },
          {
            "name": "amount",
            "type": "u64"
          }
        ]
      }
    },
    {
      "name": "tokensLocked",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "owner",
            "type": "pubkey"
          },
          {
            "name": "mint",
            "type": "pubkey"
          },
          {
            "name": "amount",
            "type": "u64"
          },
          {
            "name": "unlockTimestamp",
            "type": "i64"
          }
        ]
      }
    },
    {
      "name": "tokensWithdrawn",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "owner",
            "type": "pubkey"
          },
          {
            "name": "mint",
            "type": "pubkey"
          },
          {
            "name": "amount",
            "type": "u64"
          }
        ]
      }
    }
  ]
};
