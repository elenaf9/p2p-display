# SWP Internet-Kommunikation Projekt

Bereiche: Kommunikation Controller und Display, Kommunikation zw Controllern, Kommunikation zu Controllern.

Probleme:
- Wie lange halten die?

Notes:
- Rust kann auf RIOT laufen

## Organisation
- Paco macht ein Doodle für Termin

## Phasen
### Alpha
- Raspberry Pi
- Raspberry Pi finden sich im lokalen Netzwerk; pingen sich gegenseitig an
- Nodes müssen sich unterscheiden können zwischen kann anzeigen und nicht
- Nur Text kann angezeigt werden, wird vom Laptop geschickt

### Beta
- Eigenes Mesh-Netzwerk
- Receiver müssen identifiziert werden; Message muss zu einem node geschickt werden können
- Protokoll für Anzeige muss ausgedacht sein
- Authentifizierung; Update messages werden signiert vom Sender; public key authentifiziert sender.
- Immer noch raspberry pi

### Production
- Wenn eine Node dazu kommt, wird der Public Key mitgeteilt
- Porten auf ESP
- Web-Interface eventuell
