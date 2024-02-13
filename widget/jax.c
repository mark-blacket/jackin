#define PY_SSIZE_T_CLEAN
#include <python3.11/Python.h>
#include <stdbool.h>

#include <jack/jack.h>
#include <jack/statistics.h>

static jack_client_t * client = NULL;
static unsigned long xrun_counter;
static float xrun_delay_us;

static int cb_xrun(void *arg)
{
    xrun_counter++;
    xrun_delay_us = jack_get_xrun_delayed_usecs(client);
    return 0;
}

static PyObject * client_init(PyObject * self, PyObject * args)
{
    if (client) Py_RETURN_NONE;
    client = jack_client_open("jackin' client", JackNoStartServer, 0);
    if (!client) return PyErr_Format(PyExc_Exception, "Cannot initialize client");
    jack_set_xrun_callback(client, cb_xrun, NULL);
    if (jack_activate(client)) return PyErr_Format(PyExc_Exception, "Cannot activate client");
    Py_RETURN_NONE;
}

static PyObject * client_close(PyObject * self, PyObject * args) 
{
    if (client) {
        jack_deactivate(client);
        jack_client_close(client);
        client = NULL;
    }
    Py_RETURN_NONE;
}

static PyObject * cpu_stats(PyObject * self, PyObject * args)
{
    if (!client) return PyErr_Format(PyExc_Exception, "Client not initialized");
    return Py_BuildValue("f", jack_cpu_load(client));
}

static PyObject * xrun_stats(PyObject * self, PyObject * args)
{
    if (!client) return PyErr_Format(PyExc_Exception, "Client not initialized");
    float max_delay = jack_get_max_delayed_usecs(client);
    return Py_BuildValue("(kff)", xrun_counter, xrun_delay_us, max_delay);
}

static PyObject * xrun_reset(PyObject * self, PyObject * args)
{
    if (!client) return PyErr_Format(PyExc_Exception, "Client not initialized");
    xrun_counter = 0;
    xrun_delay_us = 0;
    jack_reset_max_delayed_usecs(client);
    Py_RETURN_NONE;
}

static PyMethodDef methods[] = {
    {"client_init",  client_init,  METH_NOARGS, "Initialize JACK client"},
    {"client_close", client_close, METH_NOARGS, "Close JACK client"},
    {"cpu_stats",    cpu_stats,    METH_NOARGS, "Get JACK's repoerted CPU usage"},
    {"xrun_stats",   xrun_stats,   METH_NOARGS, "Get a 3-element tuple with xrun count, latest xrun delay and max xrun delay"},
    {"xrun_reset",   xrun_reset,   METH_NOARGS, "Reset xrun statistics"},
    {NULL, NULL, 0, NULL}
};

static struct PyModuleDef module = { 
    PyModuleDef_HEAD_INIT, "jax", "Basic xrun monitor for JACK", -1, 
    methods, NULL, NULL, NULL, NULL
};

PyMODINIT_FUNC PyInit_jax(void)
{
    return PyModule_Create(&module);
}
