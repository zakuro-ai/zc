#include <QCoreApplication>
#include <QDebug>
#include <chrono>
#include <thread>
#include <QProcess>
#include <QFileInfo>
#include <QJsonDocument>
#include <QJsonObject>

QString DOCKER;
QString ZAKURO_HOME;
QString ZAKURO_CONF;
QString CONTEXT;

QString CONTAINER="zk0";
QString SRC="/tmp";
QString IMAGE="zakuro/node:spark";

void display(QString output){
    QTextStream out(stdout);
    out << output;
}
void sleep(int s){
    std::this_thread::sleep_for(std::chrono::milliseconds(s*1000));
}

QString exec(const QString& program, const QString& args, QString workingDirectory=SRC){
    QProcess proc;
    QStringList _args = args.split(" ");
    proc.setProcessChannelMode(QProcess::MergedChannels);
    proc.setWorkingDirectory(workingDirectory);
    proc.setProgram(program);
    proc.setArguments(_args);
    proc.start();
    proc.waitForFinished(-1);
    QString output(proc.readAll());
    proc.close();
    return output;
}


QString docker(QString arg){
    return exec(DOCKER, arg);
}
QString docker_exec(QString arg){
    return exec(DOCKER, "exec " + arg);
}

QString zcli(QString arg){
    return docker_exec(QString("-i %1 zakuro_cli ").arg(CONTAINER) + arg);
}

void down() {
    docker(QString("compose -f zakuro.yml down"));
    docker(QString("stop %1").arg(CONTAINER));
    docker(QString("rm %1").arg(CONTAINER));
}


void up() {
    docker(QString("compose -f zakuro.yml up %1 -d").arg(CONTAINER));
    sleep(5);
}

void entrypoint() {
    docker_exec("/entrypoint");
}

void restart() {
    down();
    up();

}

void logs() {
    display(zcli(QString("logs")));
}

void wg0ip() {
    display(zcli(QString("wg0ip")));

}
void nmap() {
    display(zcli(QString("nmap")));

}

void test_network() {
    logs();
    wg0ip();
    nmap();
}

void add_worker(){
    qDebug() << docker_exec(QString("-d %1 zakuro_cli add_worker").arg(CONTAINER));
}


void help(){

    qDebug("Usage:  zc [OPTIONS] COMMAND \n\
        A self-sufficient runtime for zakuro \n\
        Commands: \n\
            restart         Restart the zakuro service \n\
            *build           Build the image attached to the context (multi-platform) \n\
            *build_vanilla   Build the vanilla image attached to the context (multi-platform) \n\
            add_worker      Add a worker to the network \n\
            down            Stop the container \n\
            up              Start the container \n\
            logs            Fetch the logs of master node \n\
            pull            Pull the latest zakuro version \n\
            test_network    Test the zakuro network \n\
            ============= \n\
            *commit         Create a new image from a container's changes \n\
            *cp             Copy files/folders between a container and the local filesystem \n\
            *create         Create a new container \n\
            *exec           Run a command in a running container \n\
            *export         Export a container's filesystem as a tar archive \n\
            *images         List images \n\
            *info           Display system-wide information \n\
            *kill           Kill one or more running containers \n\
            *push           Push an image or a repository to a registry \n\
            restart         Restart the zakuro service \n\
            *rm             Remove one or more containers \n\
            *start          Start one or more stopped containers \n\
            *stop           Stop one or more running containers \n\
            *version        Show the Docker version information \n\n\
        Run 'zc COMMAND --help' for more information on a command. \n\n\
        To get more help with docker, check out our guides at https://docs.zakuro.ai/go/guides/\n"
           );
}

void load_conf(){
    QProcessEnvironment env = QProcessEnvironment::systemEnvironment();
    QString HOME = env.value("HOME");
    ZAKURO_HOME = QString("%1/.zakuro").arg(HOME);
    ZAKURO_CONF = QString("%1/config").arg(ZAKURO_HOME);
    CONTEXT = QString("%1/context.json").arg(ZAKURO_HOME);
    if (QFile::exists("/usr/local/bin/docker")){
        DOCKER="/usr/local/bin/docker";
    }
    else{
        DOCKER="/usr/bin/docker";
    }
    QString val;
    QFile file;
    file.setFileName(CONTEXT);
    file.open(QIODevice::ReadOnly | QIODevice::Text);
    val = file.readAll();
    file.close();
    QJsonDocument d = QJsonDocument::fromJson(val.toUtf8());
    QJsonObject set2 = QJsonObject(d.object());
    CONTAINER = set2.value(QString("container")).toString();
    SRC = set2.value(QString("src")).toString();
    IMAGE = set2.value(QString("image")).toString();
}

void write(){

    QJsonObject content;
    content.insert( "src", SRC );
    content.insert( "container", CONTAINER );
    content.insert( "image", IMAGE );
    QJsonDocument document;
    document.setObject( content );
    QByteArray bytes = document.toJson( QJsonDocument::Indented );
    QFile file(CONTEXT);
    if( file.open( QIODevice::WriteOnly | QIODevice::Text | QIODevice::Truncate ) )
    {
        QTextStream iStream( &file );
        iStream << bytes;
        file.close();
    }
}

void set_context(QString dir="."){
    QString val;
    QFileInfo fi(dir);
    QFile file;
    QString current_dir = fi.absoluteFilePath();
    file.setFileName(current_dir + "/zakuro.yml");
    file.open(QIODevice::ReadOnly | QIODevice::Text);
    val = file.readAll();
    QString v;
    foreach (v, val.split("\n")){
        if (v.contains("image: zakuroai/node")){
            IMAGE = v.split("image: ")[1];
        }
        else if (v.contains("container_name: "))
        {
            CONTAINER = v.split("container_name: ")[1];
        }

    }
    SRC = current_dir;
    write();
}

void pull(){
    display(docker(QString("pull %1").arg(IMAGE)));

}

void build(){
    set_context();
    display(docker(QString("buildx build --platform=linux/amd64,linux/arm64 %1 -t %2 -f %3/zk0.dockerfile --push --no-cache").arg(SRC, IMAGE, SRC)));
    pull();
}

int main(int argc, char *argv[])
{
    load_conf();
    QString arg = QString(argv[1]);
    QString arg1 = QString(argv[2]);
    if (arg=="wg0ip"){
        wg0ip();
    }
    else if(arg=="logs"){
        logs();
    }
    else if(arg=="test_network"){
        test_network();
    }
    else if(arg=="nmap"){
        nmap();
    }
    else if(arg=="restart"){
        restart();
    }
    else if(arg=="up"){
        up();
    }
    else if(arg=="add_worker"){
        add_worker();
    }
    else if(arg=="pull"){
        pull();
    }
    else if(arg=="build"){
        build();
    }
    else if(arg=="stop"){
        down();
    }
    else if(arg=="context"){
        if (arg1!=""){
            set_context(arg1);
        }
        qDebug() << SRC;
        qDebug() << CONTAINER;
        qDebug() << IMAGE;
    }
    else if(arg=="image")
    {
        display(IMAGE);
    }
    else{
        help();
    }
    return 1;
}
