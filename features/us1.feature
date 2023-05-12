Feature: User Story 1
    As a seller:
        - I create an asset with a quantity which is equal to weight in metric tons.
        - I add my product metadata to the token which can be seen by everybody.
        - I add my process and transport emissions from me to my customers in Kg CO2e emitted per metric tons of my product quantity.
        - I add upstream emissions.
        - I send the token to my buyer.

  Scenario: Seller creates an asset
    Given I have the environment prepared.

    When "Seller" creates an asset with metadata: "string" and "Upstream" emissions with the amount: 10 Grams per kilo CO2 emitted from date: 1682632800.

    Then the result should be the number "II"
