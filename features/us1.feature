Feature: User Story 1
    As a seller:
        - I create an asset with a quantity which is equal to weight in metric tons.
        - I add my product metadata to the token which can be seen by everybody.
        - I add my process and transport emissions from me to my customers in Kg CO2e emitted per metric tons of my product quantity.
        - I add upstream emissions.
        - I send the token to my buyer.
        - I verify the new owner of the token.
        - I verify the emissions are correct in the asset.

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
      "metadata": "asset metadata",
      "emission_category": "Upstream",
      "emissions": 10,
      "date": 1682632800
    }
    """

    When "Seller" transfers asset with ID 1 to "Buyer" with new "Transport" emission with the amount of 10 grams per kilo on the date 1682632800

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
          "date": 1682632800
        }
      ],
      "events": [
        {"event":{"name":"Transfer","args":["5CXgNxM5hQSk9hiKxmYsLPhGun363r4J3q98A6RtHfMZauR4","5FTrX9Po5UMmwze8Um87zjmAazxYTrWUrt61ZkTKBQ5FHbMy","1"]}},
        {"event":{"name":"Emission","args":["1","Transport",true,true,"1,682,632,800","10"]}}
      ]
    }
    """
