Feature: User Story 2
  As a buyer:
    - I want to review the product emissions which I am buying.
    - I want to trace all of the upstream emissions of an asset.

  Scenario: Buyer reviews and follows the chain of emissions for an asset
    Given A "Seller" creates an asset that is split into child assets where the asset is defined as:
    """
    {
      "metadata": "asset metadata",
      "emission_category": "Upstream",
      "emissions": 10,
      "date": 1682632800
    }
    """

    When "Buyer" performs a query on the asset with ID 3

    Then the following result should be returned
    """
    [
      [
        3,
        "0x6173736574206d65746164617461",
        [
          {
            "category": "Upstream",
            "primary": true,
            "balanced": true,
            "emissions": 3,
            "date": 1682632800
          }
        ],
        [2, 3]
      ],
      [
        2,
        "0x6173736574206d65746164617461",
        [
          {
            "category": "Upstream",
            "primary": true,
            "balanced": true,
            "emissions": 5,
            "date": 1682632800
          }
        ],
        [1, 5]
      ],
      [
        1,
        "0x6173736574206d65746164617461",
        [
          {
            "category": "Upstream",
            "primary": true,
            "balanced": true,
            "emissions": 10,
            "date": 1682632800
          }
        ],
        null
      ]
    ]
    """