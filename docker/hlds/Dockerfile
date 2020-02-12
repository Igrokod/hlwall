FROM ubuntu:18.04

RUN apt-get update
RUN apt-get install -y curl lib32gcc1 xz-utils

WORKDIR /opt/steamcmd
RUN curl -sqL "https://steamcdn-a.akamaihd.net/client/installer/steamcmd_linux.tar.gz" | tar zxvf -
RUN ./steamcmd.sh +quit # self update
RUN ./steamcmd.sh +login anonymous +force_install_dir /opt/hlds +app_update 90 validate +quit || :
RUN ./steamcmd.sh +login anonymous +force_install_dir /opt/hlds +app_update 70 validate +quit || :
RUN ./steamcmd.sh +login anonymous +force_install_dir /opt/hlds +app_update 10 validate +quit || :
RUN ./steamcmd.sh +login anonymous +force_install_dir /opt/hlds +app_update 90 validate +quit

RUN mkdir -p /root/.steam
RUN ln -s /opt/steamcmd/linux32 /root/.steam/sdk32

WORKDIR /opt/hlds
# RUN echo "10" > steam_appid.txt
CMD ["./hlds_run", "-game", "cstrike", "+maxplayers 1", "+map de_dust2", "-nomaster"]
