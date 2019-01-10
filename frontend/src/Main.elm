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
    }


type alias Item =
    { title : String
    , pubDate : Maybe String
    , link : String
    , description : String
    }


itemDecoder : Decode.Decoder Item
itemDecoder =
    Decode.map4 Item
        (Decode.field "title" Decode.string)
        (Decode.maybe (Decode.field "pub_date" Decode.string))
        (Decode.field "link" Decode.string)
        (Decode.field "description" Decode.string)


init : ( Model, Cmd Msg )
init =
    ( { url = "", items = [] }, Cmd.none )



---- UPDATE ----


type Msg
    = NoOp
    | InputURL String
    | GetRSS
    | GotRSS (Result Http.Error (List Item))


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case msg of
        NoOp ->
            ( model, Cmd.none )

        InputURL url ->
            ( { model | url = url }, Cmd.none )

        GetRSS ->
            ( model, Http.get { url = buildUrl model, expect = Http.expectJson GotRSS (Decode.list itemDecoder) } )

        GotRSS result ->
            case result of
                Ok items ->
                    ( { model | items = items }, Cmd.none )

                Err e ->
                    ( model, Cmd.none )


buildUrl : Model -> String
buildUrl model =
    "http://localhost:8080/feed?url=" ++ model.url



---- VIEW ----


view : Model -> Html Msg
view model =
    div []
        [ input [ placeholder "input RSS URL", value model.url, onInput InputURL ] []
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



---- PROGRAM ----


main : Program () Model Msg
main =
    Browser.element
        { view = view
        , init = \_ -> init
        , update = update
        , subscriptions = always Sub.none
        }
