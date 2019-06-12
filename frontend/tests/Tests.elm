module Tests exposing (suite)

import Expect exposing (Expectation)
import Fuzz exposing (Fuzzer, int, list, string)
import Main
import Test exposing (Test, describe, test)


suite : Test
suite =
    describe "backend url"
        [ test "should equal" <|
            \() ->
                let
                    ( initialValue, _ ) =
                        Main.init { backendUrl = "https://example.com/" }
                in
                Expect.equal initialValue.flags.backendUrl "https://example.com/"
        ]
