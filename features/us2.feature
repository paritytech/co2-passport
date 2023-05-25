Feature: User Story 2
  As a buyer:
    - I want to review the product emissions which I am buying.
    - I want to trace all of the upstream emissions of an asset.

  Scenario: Buyer reviews and follows the chain of emissions for an asset
    Given A "Seller" creates an asset that is split into child assets where the assets are defined as:
    """
    [
      {
        "metadata": {
          "weight": 100
        },
        "emission_category": "Upstream",
        "emissions": 15,
        "date": 1682632800
      },
      {
        "metadata": {
          "weight": 50
        },
        "emission_category": "Upstream",
        "emissions": 10,
        "date": 1682632800
      },
      {
        "metadata": {
          "weight": 25
        },
        "emission_category": "Upstream",
        "emissions": 5,
        "date": 1682632800
      }
    ]
    """

    When "Buyer" performs a query on the asset with ID 3

    Then The following result should be returned with offchain calculation of total emissions of 13.75
    """
    [
      [
        3,
        "0x7b22776569676874223a32357d",
        [
          {
            "category": "Upstream",
            "primary": true,
            "balanced": true,
            "emissions": 5,
            "date": 1682632800
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
            "date": 1682632800
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
