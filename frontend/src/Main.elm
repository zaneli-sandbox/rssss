module Main exposing (Model, Msg(..), init, main, update, view)

import Browser
import Html exposing (Attribute, Html, a, article, button, div, h1, img, input, main_, p, section, span, text)
import Html.Attributes exposing (attribute, class, disabled, href, placeholder, src, title, value)
import Html.Events exposing (keyCode, on, onClick, onInput, onMouseOver)
import Http
import Json.Decode as Decode
import RemoteData
import RemoteData exposing (RemoteData)



---- MODEL ----


type alias Flags =
    { backendUrl : String }


type alias Model =
    { inputUrl : String
    , submittedUrl : Maybe String
    , data : RemoteData String (List Item)
    , previewing : Maybe Item
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
    ( { inputUrl = ""
      , submittedUrl = Nothing
      , data = RemoteData.NotAsked
      , previewing = Nothing
      , flags = flags
      }
    , Cmd.none
    )



---- UPDATE ----


type Msg
    = NoOp
    | InputUrl String
    | GetRss
    | GotRss (RemoteData String (List Item))
    | Preview Item
    | DeletePreview


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    let
        submittedUrl data =
            if RemoteData.isSuccess data then
                Just model.inputUrl

            else
                Nothing
    in
    case msg of
        NoOp ->
            ( model, Cmd.none )

        InputUrl url ->
            ( { model | inputUrl = url }, Cmd.none )

        GetRss ->
            ( { model | data = RemoteData.Loading, previewing = Nothing }
            , Http.get { url = buildUrl model, expect = expectJson (RemoteData.fromResult >> GotRss) }
            )

        GotRss data ->
            ( { model | data = data, submittedUrl = submittedUrl data }, Cmd.none )

        Preview item ->
            ( { model | previewing = Just item }, Cmd.none )

        DeletePreview ->
            ( { model | previewing = Nothing }, Cmd.none )


expectJson : (Result String (List Item) -> msg) -> Http.Expect msg
expectJson toMsg =
    let
        decodeJson decoder body onSuccess onFailure =
            case Decode.decodeString decoder body of
                Ok value ->
                    onSuccess value

                Err e ->
                    onFailure e
    in
    Http.expectStringResponse toMsg <|
        \res ->
            case res of
                Http.GoodStatus_ _ body ->
                    decodeJson (Decode.list itemDecoder) body Ok (\e -> Decode.errorToString e |> Err)

                Http.BadStatus_ metadata body ->
                    decodeJson errDecoder body (\v -> Err v.message) (\_ -> Err metadata.statusText)

                Http.BadUrl_ url ->
                    Err "bad url"

                Http.Timeout_ ->
                    Err "timeout"

                Http.NetworkError_ ->
                    Err "network error"


buildUrl : Model -> String
buildUrl model =
    model.flags.backendUrl ++ "/feed?url=" ++ model.inputUrl


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
        invalidUrls =
            "" :: (model.submittedUrl |> Maybe.map List.singleton |> Maybe.withDefault [])

        enableOr enabled disabled =
            if RemoteData.isLoading model.data || List.member model.inputUrl invalidUrls then
                disabled

            else
                enabled

        buttonClass =
            if RemoteData.isLoading model.data then
                "button is-loading"

            else
                "button"
    in
    div [ class "level" ]
        [ div [ class "level-item" ]
            [ input
                ([ class "input"
                 , placeholder "input RSS URL"
                 , title "input RSS URL"
                 , value model.inputUrl
                 , onInput InputUrl
                 ]
                    ++ enableOr [ onEnter GetRss ] []
                )
                []
            ]
        , div [ class "level-right" ]
            [ button
                [ class buttonClass
                , enableOr (onClick GetRss) (disabled True)
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
                (RemoteData.withDefault [] model.data)
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
    let
        message =
            case model.data of
                RemoteData.Failure m ->
                    [ div [ class "notification is-danger" ] [ text m ] ]

                _ ->
                    []
    in
    div [ class "block" ] message



---- PROGRAM ----


main : Program Flags Model Msg
main =
    Browser.element
        { view = view
        , init = init
        , update = update
        , subscriptions = always Sub.none
        }
