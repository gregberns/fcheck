{ version = 2
, setup = 
  { command =
    [ { command = "rm -f ./data/cats.txt && rm -f ./data/dogs.txt" }
    ]
  }
, test = 
  [ { name = "test1"
    , description = "Write two files and check they are the same"
    , disabled = False
    , parallel = False
    , command = 
      [ { command = "rm -f ./data/cats.txt && rm ./data/cats.txt && rm -f ./data/dogs.txt" }
      , { command = "echo \"Dogs\" > ./data/dogs.txt" }
      , { command = "echo \"Dogs\" > ./data/cats.txt" }
      , { command = "diff ./data/dogs.txt ./data/cats.txt" }
      ]
    }
  ]
, teardown = 
  { command =
    [ { command = "rm -f ./data/cats.txt && rm -f ./data/dogs.txt" }
    ]
  }
}
