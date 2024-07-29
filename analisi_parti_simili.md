|     subfolder          |cosa fa                         |è utile                         |crate usati                         |
|----------------|-------------------------------|-----------------------------|-----------------------------|
|Auth|gestisce   autenticazione   e multipli viewers|  non serve autenticazione perche non è necessario autenticarci per accedere nella nostra applicazione( basta accedere all'indirizzo) , gestire multipli viewrs è utile             |tokio per gestire acessi concorrenti , per autenticazione non ho scritto perche non serve       |
|capture/audio          |gestisce input audio         | forse, se si vuole fare uno sreen sharer che condivide anche audio       | ac_ffmpeg ( usato per codifica e decodifica di aiudio), CPAL (gestisce  audio input ed output  su un dato device     |
|capture/display          | fornisce tratto display selector per selezionare che display usare         | si     | - |
|capture/macos          | gestisce cattura audio e video  su mac e screen record(stream)     | si |apple_sys( FFI( foreign fuction inteface)  bindings to C (and some C++) libraries) per macos (non valido sul lungo periodo vedi  crate core-foundation-rs per lungo supporto, ac_ffmpeg       |
|capture/wgc          |gestisce suporto per device windows            |si           |windows (permette di chiamare api di windows )         |
|capture/yuv_convert          | non chiaro , forse cambia cosa viene visualizzato su windows(colori)         |non credo     |windows   |
|capture/mod.rs         |gestisce cattura schemo ( os dipendente)    |si    |tokio , async_trait(provides an attribute macro to make async fn in traits work with dyn traits |
|capture/capturer.rs         |gestisce room , cattura display e selezione display , (atributo profiler permette streem in un file e quindi penso registrazione)  |si    |tokio , clap( parser di argomenti di linea di comando) |
|encoder          | codifica il video |si |ac_ffmpeg |
|gui         |gestisce  la gui  |si    | iced( crossplatform gui library), tokio,directories(gestione file per ogni os) |
|input      |gestisce il controllo remoto dell'altro  pc |parzialmente( solo nella parte di inserire testo forse)    |tokio,serde (serializza e deserializza strutture dati), enigo (simula input mouse e tastiera) |
|output    |gestisce  i vari tipi di output possibili ( file , stream , nessuno per testing) | si |async_trait, tokio,webrtc (gestisce protoclli web , nel nostro caso utile per far connesione e streaming),rtcp(gestisce feedback su quality of service )|
|signaller    |gestisce  le connessioni | si |async_trait, tokio,webrtc,strum(semplifica utilizzo di enum e strings ),serde |
|performance_profiler.rs |gestisce  i report sulle prestazini| no |chrono( gestisce date), howlong( cronometro per tempo di esecuzione)|
|config.rs|gestisce  configurazione iniziale probabilemnte dell'accesso al sito relativo all'app web| no |twilio (penso sia il servizio di hosting del sito web ),serde,base64,webrtc|

