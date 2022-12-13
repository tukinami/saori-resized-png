# Resized Png

[GitHub repository](https://github.com/tukinami/saori-resized-png)

## これは何?

デスクトップマスコット、「伺か」で使用できるSAORIの一種です。

機能としては、指定した画像ファイルを拡大または縮小し、pngとして出力します。

「伺か」「SAORI」等の用語については詳しく説明いたしませんのでご了承下さい。

## 使い方

SAORI自体の使い方は、使用するSHIORIなどによって異なりますので、ご自身でお調べ下さい。

ここではこのSAORIの使い方について説明いたします。

Argument0に、使用する機能名を指定して使用します。
指定できる機能は`GetImageType`と`ToResizedPng`です。

### `GetImageType`

+ Argument1: 判別するファイルのパス

+ Result: 画像形式を表す文字列

指定されたファイルの画像形式を返します。
画像でない、または対応していない画像は`UNKNOWN`が返ります。

対応している形式は以下(色深度などによっては、対応していない場合があります):

+ `AVIF`
+ `BMP`
+ `DDS`
+ `FARBFELD`
+ `GIF`
+ `HDR`
+ `ICO`
+ `JPEG`
+ `OPENEXR`
+ `PNG`
+ `PNM`
+ `TGA`
+ `TIFF`
+ `WEBP`

### `ToResizedPng`

+ Argument1: 入力するファイルのパス
+ Argument2: 出力するファイルのパス
+ Argument3: 出力する画像の横幅の数値
+ Argument4: 出力する画像の縦幅の数値

+ Result: エラーコードの数値(下記参照)

入力された画像を拡大または縮小して、pngとして出力します。
何か問題があった場合は、Resultに`0`以外が入ります。

横幅と縦幅は、負の数を指定すると、もう片方の拡大縮小率に基づいて自動で値が決まります
(両方負の数にすると、何もせずに終了します)。
また、`0`を指定すると入力された画像の値を使用します。

#### エラーコード

0. 正常終了
1. 対応していない形式だった
2. ファイルが見つからなかった
3. 入出力に問題があった
4. 画像のデコードに問題があった
5. 画像のエンコードに問題があった
6. 画像のパラメータに問題があった
7. 画像の大きさが限界値を越えていた
8. 画像サイズが小さすぎた

## 使用ライブラリ

いずれも敬称略。ありがとうございます。

+ [winapi\_rs](https://github.com/retep998/winapi-rs) / Peter Atashian
+ [encoding\_rs](https://github.com/hsivonen/encoding_rs) / Henri Sivonen
+ [image](https://github.com/image-rs/image) / The image-rs Developers
+ [fast\_image\_resize](https://github.com/cykooz/fast_image_resize) / Kirill Kuzminykh
+ (テスト実行時) [tempfile](https://github.com/Stebalien/tempfile) / Steven Allen, The Rust Project Developers, Ashley Mannix, Jason White


## ライセンス

MITにて配布いたします。

## 作成者

月波 清火 (tukinami seika) <10forchette@gmail.com>

[GitHub](https://github.com/tukinami/saori-resized-png)
