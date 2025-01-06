Condividi Schermo Rust
======================

L'app offre strumenti efficienti per la  **condivisione dello schermo** , permettendo agli utenti di visualizzare in tempo reale i contenuti condivisi, ideale per presentazioni, supporto tecnico, o sessioni di lavoro remoto.

**Funzionalità principali:**

1. **Condivisione schermo in tempo reale:**

   * Consente agli utenti di condividere lo schermo del proprio dispositivo in modo fluido e senza interruzioni.
   * Supporta schermi multipli e selezione parziale dello schermo.
   * Supporta vari dispositivi e piattaforme (Windows, MacOS e Linux).
   * L'applicazione può essere utilizzata anche per ricevere il flusso video, offrendo una soluzione completa per la condivisione dello schermo e la visualizzazione
2. **Registrazione delle sessioni:**

   * Salva le sessioni di condivisione schermo come video per uso futuro.
   * Il video viene salvato in formato FLV (si consiglia l'utilizzo di VLC Media Player).
3. **Hotkey Support:**

   * Permette scorciatoie da tastiera per eseguire funzionalità come l'interruzione, oscurazione e la pausa dello streaming.
4. **Interfaccia utente intuitiva:**

   * Design minimalista e facile da navigare per tutti i tipi di utenti.

## Installing

### Windows

Per eseguire l'applicazione è necessaria l'installazione del framework di Gstreamer:

* [Link alla guida](https://gstreamer.freedesktop.org/documentation/installing/on-windows.html?gi-language=c)
* [Link al download dei binaries](https://gstreamer.freedesktop.org/download/#windows)

L'applicativo utilizza plugin aggiuntivi oltre a quelli base per la cattura, encoding, ecc... spesso non installabili separatamente
si consiglia vivamente quindi un installazione completa del software

### Linux

Tested on Ubuntu 24.04
Per eseguire l'applicazione è necessaria l'installazione di alcuni  pacchetti, installabili nel seguente modo:

sudo apt-get install libgstreamer1.0-dev libgstreamer-plugins-base1.0-dev libgstreamer-plugins-bad1.0-dev gstreamer1.0-plugins-base gstreamer1.0-plugins-good gstreamer1.0-plugins-bad gstreamer1.0-plugins-ugly gstreamer1.0-libav gstreamer1.0-tools gstreamer1.0-x gstreamer1.0-alsa gstreamer1.0-gl gstreamer1.0-gtk3 gstreamer1.0-qt5 gstreamer1.0-pulseaudio
sudo apt-get install build-essential

## HotKeys:

CTRL + P: pausa dello streaming
CTRL + R: ravvio dello streaming
CTRL + S: inizio dello streaming
