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
            "dataSource": "Some Algorithm",
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
            "dataSource": "Some Algorithm",
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
            "dataSource": "Some Algorithm",
            "value": 5,
            "date": 1755040054
          },
            {
              "category": "Process",
              "dataSource": "Some Algorithm",
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
            "dataSource": "Some Algorithm",
            "value": 5,
            "date": 1765040054
          }
        ]
      }
    ]
    """

    When "Buyer" performs a query on the asset with ID 4

    Then The emissions can be calculated offchain for "Upstream" emissions between the dates 1700000000 and 1900000000 and weight 100 with the total equal to 1500 based on the following:
    """
    [
      {
        "assetId": 4,
        "metadata": "0x7b22776569676874223a31357d",
        "emissions": [
          {
            "category": "Process",
            "dataSource": "0x536f6d6520416c676f726974686d",
            "balanced": true,
            "value": 5,
            "date": 1765040054
          }
        ],
        "parent": 3
      },
      {
        "assetId": 3,
        "metadata": "0x7b22776569676874223a32357d",
        "emissions": [
          {
            "category": "Upstream",
            "dataSource": "0x536f6d6520416c676f726974686d",
            "balanced": true,
            "value": 5,
            "date": 1755040054
          },
          {
            "category": "Process",
            "dataSource": "0x536f6d6520416c676f726974686d",
            "balanced": true,
            "value": 5,
            "date": 1755040054
          }
        ],
        "parent": 2
      },
      {
        "assetId": 2,
        "metadata": "0x7b22776569676874223a35307d",
        "emissions": [
          {
            "category": "Upstream",
            "dataSource": "0x536f6d6520416c676f726974686d",
            "balanced": true,
            "value": 10,
            "date": 1705040054
          }
        ],
        "parent": 1
      },
      {
        "assetId": 1,
        "metadata": "0x7b22776569676874223a3130307d",
        "emissions": [
          {
            "category": "Upstream",
            "dataSource": "0x536f6d6520416c676f726974686d",
            "balanced": true,
            "value": 15,
            "date": 1682632800
          }
        ],
        "parent": null
      }
    ]
    """
