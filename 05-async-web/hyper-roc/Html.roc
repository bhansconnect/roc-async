interface Html
    exposes [
        render,
        head,
        body,
        meta,
        title,
        style,
        h1,
        p,
        ul,
        li,
        a,
    ]
    imports []

# TODO: maybe make this nicer with type checking/delayed toStr.

wrappedElems : Str, Str, List (Str -> Str) -> (Str -> Str)
wrappedElems = \start, end, elemFns ->
    \buf ->
        List.walk elemFns (Str.concat buf start) (\state, elemFn -> elemFn state)
            |> Str.concat end

render : List (Str -> Str) -> Str
render = \elemFns ->
    start = "<!DOCTYPE html><html lang=\"en\">"
    end = "</html>"
    # html is the finalizer, so it executes everything with an empty buffer.
    (wrappedElems start end elemFns) ""

head : List (Str -> Str) -> (Str -> Str)
head = \elemFns ->
    wrappedElems "<head>" "</head>" elemFns

body : List (Str -> Str) -> (Str -> Str)
body = \elemFns ->
    wrappedElems "<body>" "</body>" elemFns

meta : List {k: Str, v: Str} -> (Str -> Str)
meta = \attrs ->
    out = List.map attrs (\{k, v} -> "\(k)=\"\(v)\"")
        |> Str.joinWith " "
    \buf ->
        buf
            |> Str.concat "<meta "
            |> Str.concat out
            |> Str.concat ">"

title : Str -> (Str -> Str)
title = \val ->
    \buf ->
        buf
            |> Str.concat "<title>"
            |> Str.concat val
            |> Str.concat "</title>"

style : Str -> (Str -> Str)
style = \val ->
    \buf ->
        buf
            |> Str.concat "<style type=\"text/css\">"
            |> Str.concat val
            |> Str.concat "</style>"

h1 : Str -> (Str -> Str)
h1 = \val ->
    \buf ->
        buf
            |> Str.concat "<h1>"
            |> Str.concat val
            |> Str.concat "</h1>"

p : Str -> (Str -> Str)
p = \val ->
    \buf ->
        buf
            |> Str.concat "<p>"
            |> Str.concat val
            |> Str.concat "</p>"

# TODO: make this only take li?
ul : List (Str -> Str) -> (Str -> Str)
ul = \elemFns ->
    wrappedElems "<ul>" "</ul>" elemFns

li : (Str -> Str) -> (Str -> Str)
li = \elemFn ->
    \buf ->
        buf
            |> Str.concat "<li>"
            |> elemFn
            |> Str.concat "</li>"

a : {url: Str, text: Str} -> (Str -> Str)
a = \{url, text} ->
    \buf ->
        buf
            |> Str.concat "<a href=\""
            |> Str.concat url
            |> Str.concat "\">"
            |> Str.concat text
            |> Str.concat "</a>"
