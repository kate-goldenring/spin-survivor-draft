# Survivor Draft Spin Application

## Example from Season 46

Run application, choosing a username and password for the KV explorer

```sh
SPIN_VARIABLE_KV_EXPLORER_PASSWORD=pw SPIN_VARIABLE_KV_EXPLORER_USER=user spin watch
```

Initialize players

```sh
curl -X POST -d "Jemila,Ben,Jessica,Bhanu,Kenzie,Charlie,Liz,Jelinsky,Maria,Hunter,Moriah,Q,Soda,Randen,Tiffany,Tevin,Venus,Tim" http://127.0.0.1:3000/api/players
```

Join draft

```sh
curl -X POST -d "Maria,Tevin,Tiffany" https://survivor.fermyon.app/api/join/Kate
```

Vote out a player

```sh
curl -X POST http://127.0.0.1:3000/api/vote-out/Jelinsky
```

Leave draft

```sh
curl -X POST http://127.0.0.1:3000/api/leave/Kate  
```