Feature: User Story 2
  As a buyer:
    - I want to review the product emissions which I am buying.
    - I want to trace all of the upstream emissions of an asset.
    - I want to filter emissions based on date and emission category

  Scenario: Buyer reviews and follows the chain of emissions for an asset
    Given A "Seller" creates an asset that is split into child assets where the assets are defined as:
    """
    [
      {
        "metadata": {
          "weight": 100
        },
        "emissions": [
          {
            "emission_category": "Upstream",
            "emissions": 15,
            "date": 1682632800
          }
        ]
      },
      {
        "metadata": {
          "weight": 50
        },
        "emissions": [
          {
            "emission_category": "Upstream",
            "emissions": 10,
            "date": 1705040054
          }
        ]
      },
      {
        "metadata": {
          "weight": 25
        },
        "emissions": [
          {
            "emission_category": "Upstream",
            "emissions": 5,
            "date": 1755040054
          },
            {
              "category": "Process",
              "primary": true,
              "balanced": true,
              "emissions": 5,
              "date": 1755040054
            }
        ]
      },
      {
        "metadata": {
          "weight": 15
        },
        "emissions": [
          {
            "emission_category": "Process",
            "emissions": 5,
            "date": 1765040054
          }
        ]
      }
    ]
    """

    When "Buyer" performs a query on the asset with ID 4

    Then The emissions can be calculated offchain for "Upstream" emissions between the dates 1700000000 and 1900000000 with the total equal to 6 based on the following:
    """
    [
      [
        4,
        "0x7b22776569676874223a31357d",
        [
          {
            "category": "Process",
            "primary": true,
            "balanced": true,
            "emissions": 5,
            "date": 1765040054
          }
        ],
        [3, 15]
      ],
      [
        3,
        "0x7b22776569676874223a32357d",
        [
          {
            "category": "Upstream",
            "primary": true,
            "balanced": true,
            "emissions": 5,
            "date": 1755040054
          },
          {
            "category": "Process",
            "primary": true,
            "balanced": true,
            "emissions": 5,
            "date": 1755040054
          }
        ],
        [2, 25]
      ],
      [
        2,
        "0x7b22776569676874223a35307d",
        [
          {
            "category": "Upstream",
            "primary": true,
            "balanced": true,
            "emissions": 10,
            "date": 1705040054
          }
        ],
        [1, 50]
      ],
      [
        1,
        "0x7b22776569676874223a3130307d",
        [
          {
            "category": "Upstream",
            "primary": true,
            "balanced": true,
            "emissions": 15,
            "date": 1682632800
          }
        ],
        null
      ]
    ]
    """
