module Main exposing (Model, Msg(..), init, main, update, view)

import Browser
import Html exposing (Html, a, button, div, h1, img, input, span, text)
import Html.Attributes exposing (disabled, href, placeholder, src, title, value)
import Html.Events exposing (onClick, onInput)
import Http
import Json.Decode as Decode



---- MODEL ----


type alias Model =
    { url : String
    , items : List Item
    , message : Maybe String
    }


type alias Item =
    { title : String
    , pubDate : Maybe String
    , link : String
    , description : String
    }


type alias ResponseError =
    { message : String }


itemDecoder : Decode.Decoder Item
itemDecoder =
    Decode.map4 Item
        (Decode.field "title" Decode.string)
        (Decode.maybe (Decode.field "pub_date" Decode.string))
        (Decode.field "link" Decode.string)
        (Decode.field "description" Decode.string)


errDecoder : Decode.Decoder ResponseError
errDecoder =
    Decode.map ResponseError
        (Decode.field "message" Decode.string)


init : ( Model, Cmd Msg )
init =
    ( { url = "", items = [], message = Maybe.Nothing }, Cmd.none )



---- UPDATE ----


type Msg
    = NoOp
    | InputURL String
    | GetRSS
    | GotRSS (Result Http.Error (Http.Response String))


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case msg of
        NoOp ->
            ( model, Cmd.none )

        InputURL url ->
            ( { model | url = url }, Cmd.none )

        GetRSS ->
            ( model, Http.get { url = buildUrl model, expect = expectJson GotRSS } )

        GotRSS result ->
            case result of
                Ok res ->
                    case res of
                        Http.GoodStatus_ metadata body ->
                            case Decode.decodeString (Decode.list itemDecoder) body of
                                Ok value ->
                                    ( { model | items = value, message = Maybe.Nothing }, Cmd.none )

                                Err e ->
                                    ( { model | items = [], message = e |> Decode.errorToString |> Maybe.Just }, Cmd.none )

                        Http.BadStatus_ metadata body ->
                            case Decode.decodeString errDecoder body of
                                Ok value ->
                                    ( { model | items = [], message = Maybe.Just value.message }, Cmd.none )

                                Err _ ->
                                    ( { model | items = [], message = Maybe.Just metadata.statusText }, Cmd.none )

                        _ ->
                            ( { model | items = [], message = Maybe.Just "unexpected status" }, Cmd.none )

                Err _ ->
                    ( { model | items = [], message = Maybe.Just "invalid response" }, Cmd.none )


expectJson : (Result Http.Error (Http.Response String) -> msg) -> Http.Expect msg
expectJson toMsg =
    Http.expectStringResponse toMsg <|
        \response ->
            case response of
                Http.BadUrl_ url ->
                    Err (Http.BadUrl url)

                Http.Timeout_ ->
                    Err Http.Timeout

                Http.NetworkError_ ->
                    Err Http.NetworkError

                _ ->
                    Ok response


buildUrl : Model -> String
buildUrl model =
    "http://localhost:8080/feed?url=" ++ model.url



---- VIEW ----


view : Model -> Html Msg
view model =
    div []
        ([ input [ placeholder "input RSS URL", value model.url, onInput InputURL ] []
         , button
            [ if model.url == "" then
                disabled True

              else
                onClick GetRSS
            ]
            [ text "view RSS" ]
         , div []
            (List.map
                (\item ->
                    div []
                        [ span [] [ text (Maybe.withDefault "-" item.pubDate) ]
                        , span [ title item.description ] [ a [ href item.link ] [ text item.title ] ]
                        ]
                )
                model.items
            )
         ]
            ++ (model.message |> Maybe.map (\msg -> [ div [] [ text msg ] ]) |> Maybe.withDefault [])
        )



---- PROGRAM ----


main : Program () Model Msg
main =
    Browser.element
        { view = view
        , init = \_ -> init
        , update = update
        , subscriptions = always Sub.none
        }
