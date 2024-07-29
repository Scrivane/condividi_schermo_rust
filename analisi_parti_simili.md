|     subfolder          |cosa fa                         |è utile                         |crate usati                         |
|----------------|-------------------------------|-----------------------------|-----------------------------|
|Auth|gestisce   autenticazione   e multipli viewers|  non serve autenticazione perche non è necessario autenticarci per accedere nella nostra applicazione( basta accedere all'indirizzo) , gestire multipli viewrs è utile             |tokio per gestire acessi concorrenti , per autenticazione non ho scritto perche non serve       |
|capture/audio          |gestisce input audio         | forse, se si vuole fare uno sreen sharer che condivide anche audio       | ac_ffmpeg ( usato per codifica e decodifica di aiudio), CPAL (gestisce  audio input ed output  su un dato device     |
|capture/display          | fornisce tratto display selector per selezionare che display usare         | si     | - |
|capture/macos          | gestisce cattura audio e video  su mac e screen record(stream)     | si |apple_sys( FFI( foreign fuction inteface)  bindings to C (and some C++) libraries) per macos (non valido sul lungo periodo vedi  crate core-foundation-rs per lungo supporto, ac_ffmpeg       |
|capture/wgc          |gestisce suporto per device windows            |si           |windows (permette di chiamare api di windows )         |
|capture/yuv_convert          |`"Isn't this fun?"`            |"Isn't this fun?"            |'Isn't this fun?'            |
|Dashes          |`-- is en-dash, --- is em-dash`|-- is en-dash, --- is em-dash|'Isn't this fun?'            |
