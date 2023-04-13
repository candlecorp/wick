Feature: Serve HTTP Component

    Scenario: Start the HTTP server and make HTTP calls
        Given I have a config file at "../../../examples/serve_static_http/app.wick"
        When I run the application with "wick run ../../../examples/serve_static_http/app.wick"
        Then I can make a HTTP "POST" request on port "8999" for path "/" with body
            """
            {
                "message": "my json message"
            }
            """
        Then the response should contain
            """
            egassem nosj ym
            """
# Then I can make a HTTP "GET" request on port "8999" for path "/"
# And the response should contain "Hello World"