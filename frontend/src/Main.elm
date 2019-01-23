module Main exposing (Model, Msg(..), init, main, update, view)

import Browser
import Html exposing (Attribute, Html, a, article, button, div, h1, img, input, main_, p, section, span, text)
import Html.Attributes exposing (attribute, class, disabled, href, placeholder, src, title, value)
import Html.Events exposing (keyCode, on, onClick, onInput, onMouseOver)
import Http
import Json.Decode as Decode



---- MODEL ----


type alias Flags =
    { backendUrl : String }


type alias Model =
    { url : String
    , items : List Item
    , previewing : Maybe Item
    , message : Maybe String
    , flags : Flags
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


init : Flags -> ( Model, Cmd Msg )
init flags =
    ( { url = ""
      , items = []
      , previewing = Nothing
      , message = Nothing
      , flags = flags
      }
    , Cmd.none
    )



---- UPDATE ----


type Msg
    = NoOp
    | InputURL String
    | GetRSS
    | GotRSS (Result Http.Error ( Http.Metadata, String ))
    | Preview Item
    | DeletePreview


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case msg of
        NoOp ->
            ( model, Cmd.none )

        InputURL url ->
            ( { model | url = url }, Cmd.none )

        GetRSS ->
            ( { model | previewing = Nothing, message = Nothing }
            , Http.get { url = buildUrl model, expect = expectJson GotRSS }
            )

        GotRSS result ->
            case result of
                Ok ( metadata, body ) ->
                    let
                        isSuccess =
                            metadata.statusCode >= 200 && metadata.statusCode < 300

                        isClientError =
                            metadata.statusCode >= 400 && metadata.statusCode < 500

                        decodeJson decoder =
                            Decode.decodeString decoder body

                        invalidStatusModel =
                            { model | items = [], message = Just metadata.statusText }
                    in
                    if isSuccess then
                        case decodeJson (Decode.list itemDecoder) of
                            Ok value ->
                                ( { model | items = value }, Cmd.none )

                            Err e ->
                                ( { model | items = [], message = e |> Decode.errorToString |> Just }, Cmd.none )

                    else if isClientError then
                        case decodeJson errDecoder of
                            Ok value ->
                                ( { model | items = [], message = Just value.message }, Cmd.none )

                            Err _ ->
                                ( invalidStatusModel, Cmd.none )

                    else
                        ( invalidStatusModel, Cmd.none )

                Err _ ->
                    ( { model | items = [], message = Just "unexpected response" }, Cmd.none )

        Preview item ->
            ( { model | previewing = Just item }, Cmd.none )

        DeletePreview ->
            ( { model | previewing = Nothing }, Cmd.none )


expectJson : (Result Http.Error ( Http.Metadata, String ) -> msg) -> Http.Expect msg
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

                Http.BadStatus_ metadata body ->
                    Ok ( metadata, body )

                Http.GoodStatus_ metadata body ->
                    Ok ( metadata, body )


buildUrl : Model -> String
buildUrl model =
    model.flags.backendUrl ++ "/feed?url=" ++ model.url


onEnter : Msg -> Attribute Msg
onEnter msg =
    let
        toMsg =
            \code ->
                case code of
                    13 ->
                        msg

                    _ ->
                        NoOp
    in
    on "keypress" (keyCode |> Decode.map toMsg)



---- VIEW ----


view : Model -> Html Msg
view model =
    main_ []
        [ section [ class "container" ]
            [ inputArea model
            , feedsArea model
            , messageArea model
            ]
        ]


inputArea : Model -> Html Msg
inputArea model =
    let
        emptyOr a b =
            if model.url == "" then
                a

            else
                b
    in
    div [ class "level" ]
        [ div [ class "level-item" ]
            [ input
                ([ class "input"
                 , placeholder "input RSS URL"
                 , title "input RSS URL"
                 , value model.url
                 , onInput InputURL
                 ]
                    ++ emptyOr [] [ onEnter GetRSS ]
                )
                []
            ]
        , div [ class "level-right" ]
            [ button
                [ class "button"
                , emptyOr (disabled True) (onClick GetRSS)
                ]
                [ text "get RSS" ]
            ]
        ]


feedsArea : Model -> Html Msg
feedsArea model =
    div
        [ class "columns" ]
        [ div [ class "column" ]
            (List.map
                (\item ->
                    div [ class "columns has-text-left" ]
                        [ div [ class "column is-one-third" ] [ text (Maybe.withDefault "-" item.pubDate) ]
                        , div [ class "column", onMouseOver (Preview item) ] [ a [ href item.link ] [ text item.title ] ]
                        ]
                )
                model.items
            )
        , div [ class "column is-half" ] (model.previewing |> Maybe.map (\item -> [ div [ class "notification" ] [ previewArea item ] ]) |> Maybe.withDefault [])
        ]


previewArea : Item -> Html Msg
previewArea item =
    article [ class "message" ]
        [ div [ class "message-header" ] [ p [] [ text item.title ], button [ class "delete", attribute "aria-label" "delete", onClick DeletePreview ] [] ]
        , div [ class "message-body" ] [ text item.description ]
        ]


messageArea : Model -> Html Msg
messageArea model =
    div [ class "block" ] (model.message |> Maybe.map (\msg -> [ div [ class "notification is-danger" ] [ text msg ] ]) |> Maybe.withDefault [])



---- PROGRAM ----


main : Program Flags Model Msg
main =
    Browser.element
        { view = view
        , init = init
        , update = update
        , subscriptions = always Sub.none
        }
