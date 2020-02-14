FROM ubuntu:18.04

RUN apt-get update
RUN apt-get install -y curl lib32gcc1 xz-utils unzip

WORKDIR /opt/steamcmd
RUN curl -sqL "https://steamcdn-a.akamaihd.net/client/installer/steamcmd_linux.tar.gz" | tar zxvf -
RUN ./steamcmd.sh +quit # self update
RUN ./steamcmd.sh +login anonymous +force_install_dir /opt/hlds +app_update 90 validate +quit || :
RUN ./steamcmd.sh +login anonymous +force_install_dir /opt/hlds +app_update 70 validate +quit || :
RUN ./steamcmd.sh +login anonymous +force_install_dir /opt/hlds +app_update 10 validate +quit || :
RUN ./steamcmd.sh +login anonymous +force_install_dir /opt/hlds +app_update 90 validate +quit
RUN mkdir -p /root/.steam
RUN ln -s /opt/steamcmd/linux32 /root/.steam/sdk32

# Installing ReHlds
ENV REHLDS_URL "http://teamcity.rehlds.org/guestAuth/downloadArtifacts.html?buildTypeId=Rehlds_Publish&buildId=lastSuccessful"
WORKDIR /opt/rehlds
RUN curl -sL -o rehlds_publish.zip $REHLDS_URL
RUN unzip rehlds_publish.zip && rm rehlds_publish.zip && unzip *.zip && rm *.zip
RUN cd bin/linux32 && chmod +x hlds_linux && tar cvf - * | tar xvf - -C /opt/hlds
RUN rm -rf /opt/rehlds

# Installing Metamod-p
ENV METAMOD_P_URL "https://github.com/jkivilin/metamod-p/releases/download/v1.21p38/metamod_i686_linux_win32-1.21p38.tar.xz"
WORKDIR /opt/hlds/cstrike/addons/metamod
RUN curl -sL $METAMOD_P_URL | xzcat | tar xvf - && rm metamod.dll
COPY plugins.ini /opt/hlds/cstrike/addons/metamod/

WORKDIR /opt/hlds/cstrike/addons/reunion/
COPY reunion_0.1.92.zip .
RUN unzip -p reunion_0.1.92.zip bin/Linux/reunion_mm_i386.so > reunion_mm_i386.so
RUN unzip -p reunion_0.1.92.zip reunion.cfg > ../../reunion.cfg
RUN rm reunion_0.1.92.zip

WORKDIR /opt/hlds
RUN sed -i -- 's/gamedll_linux "dlls\/cs\.so"/gamedll_linux "addons\/metamod\/metamod.so"/g' cstrike/liblist.gam
RUN echo "10" > steam_appid.txt
CMD ["./hlds_run", "-game", "cstrike", "+maxplayers 2", "+map de_dust2", "-nomaster", "+log on", "+mp_logdetail 3", "+mp_logecho 1"]
