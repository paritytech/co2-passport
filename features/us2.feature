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
            "value": 15,
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
            "value": 10,
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
            "value": 5,
            "date": 1755040054
          },
            {
              "category": "Process",
              "balanced": true,
              "value": 5,
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
            "value": 5,
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
      {
        "assetId": 4,
        "metadata": "0x7b22776569676874223a31357d",
        "emissions": [
          {
            "category": "Process",
            "balanced": true,
            "value": 5,
            "date": 1765040054
          }
        ],
        "parent": [
          3,
          15
        ]
      },
      {
        "assetId": 3,
        "metadata": "0x7b22776569676874223a32357d",
        "emissions": [
          {
            "category": "Upstream",
            "balanced": true,
            "value": 5,
            "date": 1755040054
          },
          {
            "category": "Process",
            "balanced": true,
            "value": 5,
            "date": 1755040054
          }
        ],
        "parent": [
          2,
          25
        ]
      },
      {
        "assetId": 2,
        "metadata": "0x7b22776569676874223a35307d",
        "emissions": [
          {
            "category": "Upstream",
            "balanced": true,
            "value": 10,
            "date": 1705040054
          }
        ],
        "parent": [
          1,
          50
        ]
      },
      {
        "assetId": 1,
        "metadata": "0x7b22776569676874223a3130307d",
        "emissions": [
          {
            "category": "Upstream",
            "balanced": true,
            "value": 15,
            "date": 1682632800
          }
        ],
        "parent": null
      }
    ]
    """
