Feature: User Story 1
    As a seller:
        - I create an asset with a quantity which is equal to weight in metric tons.
        - I add my product metadata to the token which can be seen by everybody.
        - I add my process and transport emissions from me to my customers in Kg CO2e emitted per metric tons of my product quantity.
        - I add upstream emissions.
        - I send the token to my buyer.
        - I verify the new owner of the token.
        - I verify the emissions are correct in the asset.
        - I can build an asset tree by altering the original asset.

  Scenario: Seller creates an asset
    Given I have the environment prepared.

    When "Seller" blasts an asset defined as the following:
    """
    {
      "metadata": {
        "weight": 100
      },
      "emissions": [
        {
          "category": "Upstream",
          "emissions": 10,
          "primary": true,
          "balanced": true,
          "date": 1682632800
        }
      ]
    }
    """

    Then The asset 1 and emitted events will be the following:
    """
    {
      "asset": [
        1,
        "0x7b22776569676874223a3130307d",
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
      ],
      "events": [
        {"event":{"name":"Blasted","args":["1","{\"weight\":100}","5CXgNxM5hQSk9hiKxmYsLPhGun363r4J3q98A6RtHfMZauR4",null]}},
        {"event":{"name":"Emission","args":["1","Upstream",true,true,"1,682,632,800","10"]}}
      ]
    }
    """

  Scenario: Seller tranfers asset to Buyer
    Given The "Seller" has blasted the asset with the following parameters:
    """
    {
      "metadata": {
        "weight": 100
      },
      "emissions": [
        {
          "category": "Upstream",
          "emissions": 10,
          "primary": true,
          "balanced": true,
          "date": 1682632800
        }
      ]
    }
    """

    When "Seller" transfers asset with ID 1 to "Buyer" with emissions of:
    """
    [
      {
        "category": "Transport",
        "primary": true,
        "balanced": true,
        "emissions": 10,
        "date": 1702632800
      }
    ]
    """
    Then "Buyer" will be the new owner of asset 1, the emissions and transfer events will be the following:
    """
    {
      "emissions": [
        {
          "category": "Upstream",
          "primary": true,
          "balanced": true,
          "emissions": 10,
          "date": 1682632800
        },
        {
          "category": "Transport",
          "primary": true,
          "balanced": true,
          "emissions": 10,
          "date": 1702632800
        }
      ],
      "events": [
        {"event":{"name":"Transfer","args":["5CXgNxM5hQSk9hiKxmYsLPhGun363r4J3q98A6RtHfMZauR4","5FTrX9Po5UMmwze8Um87zjmAazxYTrWUrt61ZkTKBQ5FHbMy","1"]}},
        {"event":{"name":"Emission","args":["1","Transport",true,true,"1,702,632,800","10"]}}
      ]
    }
    """
  Scenario: Seller adds emissions to asset
    Given The "Seller" has blasted the following asset:
    """
    {
      "metadata": {
        "weight": 100
      },
      "emissions": [
        {
          "category": "Upstream",
          "emissions": 10,
          "primary": true,
          "balanced": true,
          "date": 1682632800
        }
      ]
    }
    """

    When "Seller" adds the following emission to the asset with ID 1:
    """
    {
      "category": "Transport",
      "primary": true,
      "balanced": true,
      "emissions": 10,
      "date": 1782632800
    }
    """

    Then The asset 1 will be:
    """
    {
      "emissions": [
        {
          "category": "Upstream",
          "primary": true,
          "balanced": true,
          "emissions": 10,
          "date": 1682632800
        },
        {
          "category": "Transport",
          "primary": true,
          "balanced": true,
          "emissions": 10,
          "date": 1782632800
        }
      ],
      "events": [
        {"event":{"name":"Emission","args":["1","Transport",true,true,"1,782,632,800","10"]}}
      ]
    }
    """

  Scenario: Seller creates asset tree
    Given The "Seller" blasts the following parent asset:
    """
    {
      "metadata": {
        "weight": 100
      },
      "emissions": [
        {
          "category": "Upstream",
          "emissions": 15,
          "primary": true,
          "balanced": true,
          "date": 1682632800
        }
      ]
    }
    """

    When "Seller" pauses the parent asset 1 and creates a child asset, which creates a child, defined as:
    """
    [
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
          }
        ]
      }
    ]
    """

    Then The asset 3 when queried will equal the following asset tree:
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
