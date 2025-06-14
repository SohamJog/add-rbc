# Running Benchmarks

This document explains how to benchmark the codebase and read benchmarks' results. It also provides a step-by-step tutorial to run benchmarks on [Amazon Web Services (AWS)](https://aws.amazon.com) accross multiple data centers (WAN).

## Setup
The core protocols are written in Rust, but all benchmarking scripts are written in Python and run with [Fabric](http://www.fabfile.org/). To run the remote benchmark, install the python dependencies:

```
$ pip install -r requirements.txt
```

You also need to install [tmux](https://linuxize.com/post/getting-started-with-tmux/#installing-tmux) (which runs all nodes and clients in the background). 

## AWS Benchmarks
This repo integrates various python scripts to deploy and benchmark the codebase on [Amazon Web Services (AWS)](https://aws.amazon.com). They are particularly useful to run benchmarks in the WAN, across multiple data centers. This section provides a step-by-step tutorial explaining how to use them.

### Step 1. Set up your AWS credentials
Set up your AWS credentials to enable programmatic access to your account from your local machine. These credentials will authorize your machine to create, delete, and edit instances on your AWS account programmatically. First of all, [find your 'access key id' and 'secret access key'](https://docs.aws.amazon.com/cli/latest/userguide/cli-configure-quickstart.html#cli-configure-quickstart-creds). Then, create a file `~/.aws/credentials` with the following content:
```
[default]
aws_access_key_id = YOUR_ACCESS_KEY_ID
aws_secret_access_key = YOUR_SECRET_ACCESS_KEY
```
Do not specify any AWS region in that file as the python scripts will allow you to handle multiple regions programmatically.

### Step 2. Add your SSH public key to your AWS account
You must now [add your SSH public key to your AWS account](https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/ec2-key-pairs.html). This operation is manual (AWS exposes little APIs to manipulate keys) and needs to be repeated for each AWS region that you plan to use. Upon importing your key, AWS requires you to choose a 'name' for your key; ensure you set the same name on all AWS regions. This SSH key will be used by the python scripts to execute commands and upload/download files to your AWS instances.
If you don't have an SSH key, you can create one using [ssh-keygen](https://www.ssh.com/ssh/keygen/):
```
$ ssh-keygen -f ~/.ssh/aws
```

### Step 3. Configure the testbed
The file [settings.json](https://github.com/SohamJog/add-rbc/blob/main/benchmark/settings.json) (located in [add-rbc/benchmark](https://github.com/SohamJog/add-rbc/blob/main/benchmark)) contains all the configuration parameters of the testbed to deploy. Its content looks as follows:
```json
{
    "key": {
        "name": "aws",
        "path": "/absolute/key/path"
    },
    "port": 8500,
    "client_base_port": 9000,
    "client_run_port": 9500,
    "repo": {
        "name": "add-rbc",
        "url": "https://github.com/SohamJog/add-rbc",
        "branch": "master"
    },
    "instances": {
        "type": "t3a.medium",
        "regions": ["us-east-1","us-east-2","us-west-1","us-west-2","ca-central-1", "eu-west-1", "ap-southeast-1", "ap-northeast-1"]
    }
}
```
The first block (`key`) contains information regarding your SSH key:
```json
"key": {
    "name": "aws",
    "path": "/absolute/key/path"
},
```
Enter the name of your SSH key; this is the name you specified in the AWS web console in step 2. Also, enter the absolute path of your SSH private key (using a relative path won't work). 


The second block (`ports`) specifies the TCP ports to use:
```json
"port": 8500,
"client_base_port": 9000,
"client_run_port": 9500,
```
The artifact requires a number of TCP ports for communication between the processes. Note that the script will open a large port range (5000-10000) to the WAN on all your AWS instances. 

The third block (`repo`) contains the information regarding the repository's name, the URL of the repo, and the branch containing the code to deploy: 
```json
"repo": {
    "name": "add-rbc",
    "url": "https://github.com/SohamJog/add-rbc",
    "branch": "main"
},
```
Remember to update the `url` field to the name of your repo. Modifying the branch name is particularly useful when testing new functionalities without having to checkout the code locally. 

The the last block (`instances`) specifies the [AWS instance type](https://aws.amazon.com/ec2/instance-types) and the [AWS regions](https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/using-regions-availability-zones.html#concepts-available-regions) to use:
```json
"instances": {
    "type": "t3a.medium",
    "regions": ["us-east-1","us-east-2","us-west-1","us-west-2","ca-central-1", "eu-west-1", "ap-southeast-1", "ap-northeast-1"]
}
```
The instance type selects the hardware on which to deploy the testbed. For example, `t3a.medium` instances come with 2 vCPU (2 physical cores), and 4 GB of RAM. The python scripts will configure each instance with 300 GB of SSD hard drive. The `regions` field specifies the data centers to use. If you require more nodes than data centers, the python scripts will distribute the nodes as equally as possible amongst the data centers. All machines run a fresh install of Ubuntu Server 20.04.

### Step 4. Set up experiment parameters
You can edit `BENCH_PARAMETERS` in `fabfile.py` to change parameters like the number of nodes, byzantine behavior, crash behavior etc.

### Step 5. Create a testbed
The AWS instances are orchestrated with [Fabric](http://www.fabfile.org) from the file [fabfile.py](https://github.com/SohamJog/add-rbc/blob/main/benchmark/fabfile.py) (located in [add-rbc/benchmark](https://github.com/SohamJog/add-rbc/blob/main/benchmark)); you can list all possible commands as follows:
```
$ cd add-rbc/benchmark
$ fab --list
```
The command `fab create` creates new AWS instances; open [fabfile.py](https://github.com/SohamJog/add-rbc/blob/main/benchmark/fabfile.py) and locate the `create` task:
```python
@task
def create(ctx, nodes=2):
    ...
```
The parameter `nodes` determines how many instances to create in *each* AWS region. That is, if you specified 8 AWS regions as in the example of step 3, setting `nodes=2` will creates a total of 16 machines:
```
$ fab create

Creating 16 instances |██████████████████████████████| 100.0% 
Waiting for all instances to boot...
Successfully created 16 new instances
```
You can then clone the repo and install rust on the remote instances with `fab install`:
```
$ fab install

Installing rust and cloning the repo...
Initialized testbed of 16 nodes
```
This may take a long time as the command will first update all instances.
The commands `fab stop` and `fab start` respectively stop and start the testbed without destroying it (it is good practice to stop the testbed when not in use as AWS can be quite expensive); and `fab destroy` terminates all instances and destroys the testbed. Note that, depending on the instance types, AWS instances may take up to several minutes to fully start or stop. The command `fab info` displays a nice summary of all available machines and information to manually connect to them (for debug).

### Step 6. Run a benchmark
After setting up the testbed, running a benchmark on AWS is similar to running it locally. Locate the task `remote` in [fabfile.py](https://github.com/SohamJog/add-rbc/blob/main/benchmark/fabfile.py):
```python
@task
def remote(ctx):
    ...
```
Change the number of nodes to run in the `remote` function. Run the benchmark with the following command. 
```
$ fab remote
```
This command first updates all machines with the latest commit of the GitHub repo and branch specified in your file [settings.json](https://github.com/SohamJog/add-rbc/blob/main/benchmark/settings.json) (step 3); this ensures that benchmarks are always run with the latest version of the code. It then generates and uploads the configuration files to each machine, and runs the benchmarks with the specified parameters. Make sure to change the number of nodes in the `remote` function. The input parameters for add-rbc can be set in the `_config` function in the benchmark/remote.py file in the `benchmark` folder. 


### Step 7: Download logs
The following command downloads the log file from the `syncer` titled `syncer.log`. 
```
$ fab logs
```
The `syncer.log` file contains the details about the latency of the protocol and the outputs of the nodes. Note that this log file needs to be downloaded only after allowing the protocol sufficient time to terminate (Ideally within 5 minutes). If anything goes wrong during a benchmark, you can always stop it by running `fab kill`.

Be sure to kill the prior benchmark using the following command before running a new benchmark. 
```
$ fab kill
```

If you want to run the experiment with the same parameters multiple times, you can run `fab rerun` so that you don't have to set up all the machines multiple times.
```
$ fab rerun
```

### Running the benchmark for different numbers of nodes
After running the benchmarks for a given number of nodes, destroy the testbed with the following command. 
```
$ fab destroy
```
This command destroys the testbed and terminates all created AWS instances.


# Reproducing results
We ran add-rbc at with $n=16,40$ nodes, with and without crash and byzantine faults (see step 4 to tweak parameters) in a geo-distributed testbed of `t3a.medium` nodes spread across 8 regions: "us-east-1","us-east-2","us-west-1","us-west-2","ca-central-1", "eu-west-1", "ap-southeast-1", "ap-northeast-1" (These values are pre-configured in the `settings.json` file).

We ran ctrbc and Bracha's rbc with the same configuration for comparison.

# Artifact Evaluation on Cloudlab/Custom testbeds
It is possible to evaluate our artifact on CloudLab/Chameleon. However, it would require us to change a few lines of code in the submitted artifact. The benchmarking code in the current artifact works in the following way.

1. It takes a user's AWS credentials and uses the AWS boto3 SDK to spawn AWS EC2 machines across the specified regions. 
2. It also establishes a network between them using the boto3 SDK. 
3. It gets the IP addresses of the spawned machines and installs the artifact in each machine using `tmux` and `SSH`. 
4. It then runs the artifact by executing a series of commands on the machines using `tmux` and `SSH`.

We describe the series of modifications to this structure to run benchmarks on Cloudlab/Chameleon. 
## Setting up the testbed
1. Running the benchmark on Cloudlab or Chameleon requires you to skip the first two steps and create machines manually. Therefore, instead of running `fab create` and `fab start` commands, create machines manually on Cloudlab/Chameleon, and establish a network between them. This network should enable processes on the machines to communicate with each other through `TCP`. 

## Installing the Artifact
2. The `hosts()` function in the file `benchmark/benchmark/instance.py` is responsible for configuring hosts in the network. We changed the function to the following for evaluation on custom testbeds. In case the code needs to be run on AWS, uncomment the commented part and comment the uncommented part of the `hosts` function. 
```
# To run on CloudLab/Chameleon, create a list of ip addresses and add them to a file titled 'instance_ips'.
def hosts(self, flat=False):
    import json
    with open("instance-ips.json") as json_file:
        json_data = json.load(json_file)
        if flat:
            return [x for y in json_data.values() for x in y]        
        else:
            return json_data
    #try:
    #    _, ips = self._get(['pending', 'running'])
    #    return [x for y in ips.values() for x in y] if flat else ips
    #except ClientError as e:
    #    raise BenchError('Failed to gather instances IPs', AWSError(e))
```
3. Then, create a file with the name `instance-ips.json` in the `benchmark/` directory. The file should have the following structure. The key of each item in the map should be the location where the nodes are located, and the value is an array of ip addresses in that region. The benchmark distributes processes evenly in machines across different regions. In case all nodes are located in one region, use one key to list all the ip addresses. **Note that the total number of ip addresses listed must be at least as much as the number of processes being run in the benchmark. To run multiple processes on a single machine, list the ip address multiple times in the array. For example, to run two processes on the machine with ip `10.43.0.231`, list it twice in the array as ["10.43.0.231","10.43.0.231",..].** We suggest giving each process 2 CPU cores and 4 GigaBytes of RAM. For example, if you have a machine with 8 cores and 16 GB of RAM, you can run four processes in it by listing its ip address four times in the array. 
```
{
    "Utah": ["10.43.0.231",”10.43.0.231”,"10.43.0.232","10.43.0.233"],
    "Wisconsin": ["10.43.0.234","10.43.0.235","10.43.0.236"]
}
```
4. Next, the code requires access to the machines on CloudLab/Chameleon. We used the `paramiko` authentication library in Python to remotely access the machines. 
You need to specify the required SSH key in the `settings.json` file in the `benchmark` folder. 
Further, the ports specified in the `settings.json` file should be open for communication in the spawned machines. 
Finally, the username in the file `remote.py` should be changed at 8 occurrences. We hardcoded the username `ubuntu` in the file `remote.py` (We apologize for this inconvenience). Change it to the appropriate username. (Leave it as is if the machines have Ubuntu OS). 
5. The configuration in `fabfile.py` needs to be changed to run the benchmark with the appropriate number of nodes. After this change, install the required dependencies to run the code in the `benchmark` folder. Pertinent instructions have been given in `benchmark/README.md` file. Then, run `fab install` to install the artifact in all the machines. Ensure that the machines have access to the internet to help access the dependencies necessary for installation. 
6. Finally, follow the instructions in the `benchmark/README.md` file from Step 5 to run the benchmarks and plot results. 

