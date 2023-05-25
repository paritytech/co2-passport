Feature: User Story 1
    As a seller:
        - I create an asset with a quantity which is equal to weight in metric tons.
        - I add my product metadata to the token which can be seen by everybody.
        - I add my process and transport emissions from me to my customers in Kg CO2e emitted per metric tons of my product quantity.
        - I add upstream emissions.
        - I send the token to my buyer.

  Scenario: Seller creates an asset
    Given I have the environment prepared.

    When "Seller" creates an asset with metadata: "asset metadata" and "Upstream" emissions with the amount: 10 Grams per kilo CO2 emitted from date: 1682632800.

    Then the following events will be emitted:
    """
    [
      {"event":{"name":"Blasted","args":["1","asset metadata","5CXgNxM5hQSk9hiKxmYsLPhGun363r4J3q98A6RtHfMZauR4",null]}},
      {"event":{"name":"Emission","args":["1","Upstream",true,true,"1,682,632,800","10"]}}
    ]
    """

  Scenario: Seller tranfers asset to Buyer
    Given the "Seller" has blasted the asset with the following parameters:
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
