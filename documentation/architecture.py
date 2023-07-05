from diagrams import Cluster, Diagram, Edge
from diagrams.onprem.monitoring import Prometheus, Grafana
from diagrams.onprem.database import MariaDB
from diagrams.k8s.network import Ingress, Service
from diagrams.k8s.compute import Deployment, Job
from diagrams.programming.language import Rust

graph_attr = {
    "splines":"curved",
    "layout":"neato",
}


with Diagram("Lifecycle Manager", show=True):
    metrics = Prometheus("metric")
    metrics << Edge(color="firebrick", style="dashed") << Grafana("monitoring")
    service = Service("svc")
    ingress = Ingress("domain.com") >> service
    db = MariaDB("Database")

    with Cluster("Internal API"):
        Rust(label="", fontsize="6",
            fixedsize="true", width="0.5", height="0.8", 
            pin="true", pos="0,0")
        api = Deployment("lifecycle manager")
        api - db << Edge(label="collect")


    with Cluster("K8s Worker Jobs"):
        workers = [
            Job("worker[02]"),
            Job("worker[n]"),
            Job("worker[01]")]

    Service("internal_svc") >> api >> workers
    api << metrics


    with Cluster("Customer"):
        api = Deployment("customer manager")
        api - db << Edge(label="collect")

    ingress >> api
    api << metrics